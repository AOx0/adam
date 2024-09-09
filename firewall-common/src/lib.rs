#![no_std]

#[derive(Debug, Clone, Copy)]
pub enum FirewallEvent {
    Blocked([u8; 4]),
    Pass,
}
