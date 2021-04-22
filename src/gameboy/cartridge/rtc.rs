use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The Clock Counter Registers
///  08h  RTC S   Seconds   0-59 (0-3Bh)
///  09h  RTC M   Minutes   0-59 (0-3Bh)
///  0Ah  RTC H   Hours     0-23 (0-17h)
///  0Bh  RTC DL  Lower 8 bits of Day Counter (0-FFh)
///  0Ch  RTC DH  Upper 1 bit of Day Counter, Carry Bit, Halt Flag
///        Bit 0  Most significant bit of Day Counter (Bit 8)
///        Bit 6  Halt (0=Active, 1=Stop Timer)
///        Bit 7  Day Counter Carry Bit (1=Counter Overflow)
#[derive(Debug)]
pub struct RealTimeClock {
    s: u8,
    m: u8,
    h: u8,
    dl: u8,
    dh: u8,
    zero: u64,
    sav_path: PathBuf,
    is_locked: bool,
}

impl RealTimeClock {
    pub fn new(sav_path: impl AsRef<Path>) -> Self {
        let zero = match std::fs::read(sav_path.as_ref()) {
            Ok(ok) => {
                let mut b: [u8; 8] = Default::default();
                b.copy_from_slice(&ok);
                u64::from_be_bytes(b)
            }
            Err(_) => SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        Self {
            zero,
            s: 0,
            m: 0,
            h: 0,
            dl: 0,
            dh: 0,
            sav_path: sav_path.as_ref().to_path_buf(),
            is_locked: false,
        }
    }

    #[inline]
    pub fn lock(&mut self) {
        self.is_locked = true;
    }

    #[inline]
    pub fn unlock(&mut self) {
        self.is_locked = false;
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    pub fn tick(&mut self) {
        let d = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.zero;

        self.s = (d % 60) as u8;
        self.m = (d / 60 % 60) as u8;
        self.h = (d / 3600 % 24) as u8;
        let days = (d / 3600 / 24) as u16;
        self.dl = (days % 256) as u8;
        match days {
            0x0000..=0x00ff => {}
            0x0100..=0x01ff => {
                self.dh |= 0x01;
            }
            _ => {
                self.dh |= 0x01;
                self.dh |= 0x80;
            }
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, a: u16) -> u8 {
        match a {
            0x08 => self.s,
            0x09 => self.m,
            0x0a => self.h,
            0x0b => self.dl,
            0x0c => self.dh,
            _ => panic!("Invalid item"),
        }
    }

    #[allow(dead_code)]
    pub fn set(&mut self, a: u16, v: u8) {
        match a {
            0x08 => self.s = v,
            0x09 => self.m = v,
            0x0a => self.h = v,
            0x0b => self.dl = v,
            0x0c => self.dh = v,
            _ => panic!("Invalid item"),
        }
    }
}

impl Drop for RealTimeClock {
    fn drop(&mut self) {
        if self.sav_path.to_str().unwrap().is_empty() {
            return;
        }
        File::create(self.sav_path.clone())
            .and_then(|mut f| f.write_all(&self.zero.to_be_bytes()))
            .unwrap()
    }
}
