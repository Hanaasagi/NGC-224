use log::warn;

// Gameboy hardware specifications
pub const SCREEN_W: usize = 160;
pub const SCREEN_H: usize = 144;

pub const CLOCK_FREQUENCY: u32 = 4_194_304;
pub const STEP_TIME: u32 = 16;
pub const STEP_CYCLES: u32 = (STEP_TIME as f64 / (1000_f64 / CLOCK_FREQUENCY as f64)) as u32;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Term {
    GB,  // Original GameBoy (GameBoy Classic)
    GBP, // GameBoy Pocket/GameBoy Light
    GBC, // GameBoy Color
    SGB, // Super GameBoy
}

static mut NOW_TERM: Term = Term::GB;

pub fn get_global_term() -> Term {
    unsafe { NOW_TERM }
}

pub fn set_global_term(t: Term) {
    unsafe {
        warn!(
            "Change the Emulator from {:?} to {:?}, it will affect global",
            NOW_TERM, t
        );
        NOW_TERM = t
    }
}
