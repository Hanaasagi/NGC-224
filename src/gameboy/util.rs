/// Returns `true` if the bit is 1. It is starts from zero.
#[inline]
pub fn test_bit(v: u8, b: u8) -> bool {
    assert!(b <= 7);
    v & (1 << b) != 0
}

/// modify the bit to 0.
#[inline]
pub fn clear_bit(v: u8, b: u8) -> u8 {
    assert!(b <= 7);

    v & !(1 << b)
}

/// modify the bit to 1.
#[inline]
pub fn set_bit(v: u8, b: u8) -> u8 {
    assert!(b <= 7);
    v | (1 << b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_bit() {
        assert!(clear_bit(0b0011, 1) == 0b0001);
    }
    #[test]
    fn test_test_bit() {
        assert!(test_bit(0b0011, 1) == true);
    }

    #[test]
    fn test_set_bit() {
        assert!(set_bit(0b0011, 3) == 0b1011);
    }
}
