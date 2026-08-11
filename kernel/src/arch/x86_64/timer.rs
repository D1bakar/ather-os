//! Programmable Interval Timer (PIT) channel 0 on IRQ 0.
//!
//! Uses the legacy 8254 PIT in square-wave mode to generate periodic timer
//! interrupts at approximately 100 Hz.

use super::pic;
use super::ports::outb;

const PIT_CMD: u16 = 0x43;
const PIT_CH0: u16 = 0x40;

/// Nominal PIT input frequency in Hz.
pub const PIT_BASE_HZ: u32 = 1_193_182;

/// Target timer interrupt rate for M2.
pub const TIMER_HZ: u32 = 100;

/// Hardware IRQ line used by PIT channel 0.
pub const TIMER_IRQ: u8 = 0;

/// CPU vector for the timer interrupt after PIC remapping.
pub const TIMER_VECTOR: u8 = pic::PIC_VECTOR_OFFSET + TIMER_IRQ;

/// Effective interrupt rate with integer PIT divisor rounding.
#[allow(dead_code)]
pub const EFFECTIVE_TIMER_HZ: u32 = PIT_BASE_HZ / (PIT_BASE_HZ / TIMER_HZ);

/// Programs PIT channel 0 and unmasks IRQ 0.
pub fn init() {
    let divisor = (PIT_BASE_HZ / TIMER_HZ) as u16;

    unsafe {
        // Channel 0, lobyte/hibyte access, mode 3 (square wave), binary.
        outb(PIT_CMD, 0x36);
        outb(PIT_CH0, (divisor & 0xFF) as u8);
        outb(PIT_CH0, ((divisor >> 8) & 0xFF) as u8);
    }

    pic::unmask(TIMER_IRQ);
}
