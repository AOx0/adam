#![no_std]
#![no_main]

use core::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::{Array, RingBuf},
    programs::XdpContext,
};
use aya_log_ebpf::{error, info};
use firewall_common::{Direction, FirewallAction, FirewallEvent, FirewallMatch, FirewallRule};
use netp::{
    aya::XdpErr,
    bounds,
    link::{EtherType, Ethernet},
    network::IPv4,
};

#[map]
static FIREWALL_EVENTS: RingBuf = RingBuf::with_byte_size(4096, 0);

#[map]
static FIREWALL_RULES: Array<FirewallRule> = Array::with_max_entries(100, 0);

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

    bounds!(ctx, Ethernet::MIN_LEN).or_drop()?;
    bounds!(ctx, Ethernet::MIN_LEN + IPv4::MIN_LEN).or_drop()?;
    let (eth, rem) = Ethernet::new(packet).or_pass()?;

    // TODO: Impl this
    // if let EtherType::IPv6 = eth.ethertype() {}

    if let EtherType::IPv4 = eth.ethertype() {
        bounds!(ctx, eth.size_usize() + IPv4::MIN_LEN).or_drop()?;
        let (ip4, rem): (IPv4<&[u8]>, &[u8]) = IPv4::new(rem).or_drop()?;

        // while let Some(rule) = FIREWALL_RULES.get(i) {
        let rule = FIREWALL_RULES.get(0).ok_or(0).or_pass()?;

        bounds!(ctx, eth.size_usize() + IPv4::MIN_LEN).or_drop()?;
        let matching_ip = if rule.applies_to == Direction::Source {
            ip4.source_u32()
        } else {
            ip4.destination_u32()
        };
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_bits(matching_ip)), 0);

        if let FirewallMatch::Protocol(p) = rule.matches {
            bounds!(ctx, eth.size_usize() + IPv4::MIN_LEN).or_drop()?;
            let protocol = ip4.protocol();

            if protocol == p {
                return emit(ctx, rule.action, Some(socket_addr));
            }
        }

        if let FirewallMatch::Match(core::net::IpAddr::V4(addr)) = rule.matches {
            if addr.to_bits() == matching_ip {
                return emit(ctx, rule.action, Some(socket_addr));
            }
        }

        // i += 1;
        // }
    }

    emit(ctx, FirewallAction::Accept, None)
}

fn emit(ctx: XdpContext, action: FirewallAction, socket: Option<SocketAddr>) -> Result<u32, u32> {
    if let Some(mut entry) = FIREWALL_EVENTS.reserve::<FirewallEvent>(0) {
        unsafe {
            core::ptr::write_unaligned(
                entry.as_mut_ptr(),
                match action {
                    FirewallAction::Accept => FirewallEvent::Pass,
                    FirewallAction::Drop => {
                        if let Some(s) = socket {
                            FirewallEvent::Blocked(s)
                        } else {
                            FirewallEvent::Pass
                        }
                    }
                },
            )
        };

        entry.submit(0);
    } else {
        error!(&ctx, "Failed to reserve entry for FirewallEvent")
    }

    Ok(action.into())
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
