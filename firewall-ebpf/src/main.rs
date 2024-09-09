#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::RingBuf,
    programs::XdpContext,
};
use aya_log_ebpf::error;
use firewall_common::FirewallEvent;

#[map]
static FIREWALL_EVENTS: RingBuf = RingBuf::with_byte_size(4096, 0);

#[xdp]
pub fn firewall(ctx: XdpContext) -> u32 {
    match try_firewall(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_firewall(ctx: XdpContext) -> Result<u32, u32> {
    if let Some(mut entry) = FIREWALL_EVENTS.reserve::<FirewallEvent>(0) {
        unsafe { core::ptr::write_unaligned(entry.as_mut_ptr(), FirewallEvent::Pass) };

        entry.submit(0);
    } else {
        error!(&ctx, "Failed to reserve entry for FirewallEvent")
    }

    Ok(xdp_action::XDP_PASS)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
