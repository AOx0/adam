#![no_std]
#![no_main]
#![feature(let_chains)]

use core::net::{IpAddr, Ipv4Addr, SocketAddr};

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::{Array, ProgramArray, RingBuf},
    programs::XdpContext,
};
use aya_log_ebpf::error;

use firewall_common::{processor, Action, Direction, Event, Match, Rule, MAX_RULES};
use netp::{
    aya::XdpErr,
    bounds,
    link::{EtherType, Ethernet},
    network::IPv4,
    transport::tcp::Tcp,
};

#[map]
static PROCESSOR: ProgramArray = ProgramArray::with_max_entries(50, 0);

#[map]
static FIREWALL_EVENTS: RingBuf = RingBuf::with_byte_size(4096, 0);

#[map]
static FIREWALL_RULES: Array<Rule> = Array::with_max_entries(MAX_RULES, 0);

#[xdp]
pub fn firewall(ctx: XdpContext) -> u32 {
    match try_firewall(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_firewall(ctx: XdpContext) -> Result<u32, u32> {
    let packet = unsafe {
        core::slice::from_raw_parts_mut(ctx.data() as *mut u8, ctx.data_end() - ctx.data())
    };

    bounds!(ctx, 50).or_pass()?;
    let (eth, rem) = Ethernet::new(packet).or_pass()?;

    // TODO: Impl this
    // if let EtherType::IPv6 = eth.ethertype() {}

    if let EtherType::IPv4 = eth.ethertype() {
        let (ip4, _): (IPv4<&[u8]>, &[u8]) = IPv4::new(rem).or_drop()?;

        for i in 0..MAX_RULES {
            let Some(
                rule @ Rule {
                    init: true,
                    enabled: true,
                    ..
                },
            ) = FIREWALL_RULES.get(i)
            else {
                continue;
            };

            // info!(&ctx, "try_firewall: Verifying rule on {}", rule.id);

            let matching_ip = if rule.applies_to == Direction::Source {
                ip4.source_u32()
            } else {
                ip4.destination_u32()
            };
            let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_bits(matching_ip)), 0);

            if let Match::Protocol(p) = rule.matches {
                let protocol = ip4.protocol().or_drop()?;

                if protocol == p {
                    return emit(ctx, rule.action, Some((i, socket_addr)));
                }
            }

            if let Match::Match(core::net::IpAddr::V4(addr)) = rule.matches {
                if addr.to_bits() == matching_ip {
                    return emit(ctx, rule.action, Some((i, socket_addr)));
                }
            }

            if let Match::BytesAtPosition { position, value } = rule.matches {
                if rem.get(position).copied() == Some(value) {
                    return emit(ctx, rule.action, Some((i, socket_addr)));
                }
            }
        }

        unsafe { PROCESSOR.tail_call(&ctx, processor::IPV4_TCP).or_drop()? };
    }

    emit(ctx, Action::Accept, None)
}

#[xdp]
pub fn ipv4_tcp(ctx: XdpContext) -> u32 {
    match try_ipv4_tcp(ctx) {
        Ok(c) => c,
        Err(c) => c,
    }
}

/// This must be called only when IPV4 + Tcp
fn try_ipv4_tcp(ctx: XdpContext) -> Result<u32, u32> {
    let packet = unsafe {
        core::slice::from_raw_parts_mut(ctx.data() as *mut u8, ctx.data_end() - ctx.data())
    };

    bounds!(ctx, 50).or_pass()?;
    let (eth, rem) = Ethernet::new(packet).or_pass()?;
    let (ip4, rem) = IPv4::new(rem).or_drop()?;

    bounds!(ctx, eth.size_usize() + ip4.size_usize() + Tcp::MIN_LEN).or_drop()?;
    let (tcp, _) = Tcp::new(rem).or_drop()?;

    // Avoid branching as much as possible
    bounds!(ctx, Ethernet::MIN_LEN + IPv4::MIN_LEN + Tcp::MIN_LEN).or_drop()?;
    let source_ip = ip4.source_u32();
    let dest_ip = ip4.destination_u32();

    // Avoid branching as much as possible
    bounds!(ctx, eth.size_usize() + ip4.size_usize() + Tcp::MIN_LEN).or_drop()?;
    let source = tcp.source();
    let dest = tcp.destination();

    for i in 0..MAX_RULES {
        let Some(
            rule @ Rule {
                init: true,
                enabled: true,
                ..
            },
        ) = FIREWALL_RULES.get(i)
        else {
            continue;
        };

        // info!(&ctx, "try_ipv4_tcp: Verifying rule on {}", rule.id);

        let (ip, port) = if rule.applies_to == Direction::Source {
            (source_ip, source)
        } else {
            (dest_ip, dest)
        };

        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_bits(ip)), port);

        if let Match::Port(rule_port) = rule.matches {
            if rule_port == port {
                return emit(ctx, rule.action, Some((i, socket_addr)));
            }
        }
    }

    emit(ctx, Action::Accept, None)
}

fn emit(ctx: XdpContext, action: Action, socket: Option<(u32, SocketAddr)>) -> Result<u32, u32> {
    if let Some(mut entry) = FIREWALL_EVENTS.reserve::<Event>(0) {
        unsafe {
            core::ptr::write_unaligned(
                entry.as_mut_ptr(),
                match action {
                    Action::Accept => Event::Pass,
                    Action::Drop => {
                        if let Some((rule, addr)) = socket {
                            Event::Blocked { rule, addr }
                        } else {
                            Event::Pass
                        }
                    }
                },
            )
        };

        entry.submit(0);
    } else {
        error!(&ctx, "Failed to reserve entry for Event")
    }

    Ok(action.into())
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
