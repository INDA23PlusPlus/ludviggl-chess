
const FILL: u64 = 0xffffffffffffffff;

pub fn flatten(x: u8, y: u8) -> usize {
    (x | (y << 3)) as usize
}

pub fn bit(b: u64) -> u64 {
    1 << b
}

pub fn shr_unchecked(x: u64, s: u64) -> u64 {
    x.checked_shr(s.try_into().unwrap()).unwrap_or(0)
}

pub fn shl_unchecked(x: u64, s: u64) -> u64 {
    x.checked_shl(s.try_into().unwrap()).unwrap_or(0)
}

pub fn flatten_bit(x: u8, y: u8) -> u64 {
    bit(flatten(x, y) as u64)
}

pub fn unflatten(i: usize) -> (u8, u8) {
    ((i & 7) as u8, (i >> 3) as u8)
}

pub fn unflatten_bit(m: u64) -> (u8, u8) {
    unflatten(m.trailing_zeros() as usize)
}

// Fills bits left of ls 1 of m, incl ls 1
// FIlls all if m == 0
pub fn fill_left_incl(m: u64) -> u64 {
    if m > 0 { shl_unchecked(FILL, m.trailing_zeros().into()) }
    else { FILL }
}

pub fn fill_left_excl(m: u64) -> u64 {
    if m > 0 { shl_unchecked(FILL, (m.trailing_zeros() + 1).into()) }
    else { FILL }
}

pub fn fill_right_incl(m: u64) -> u64 {
    if m > 0 { shr_unchecked(FILL, m.leading_zeros().into()) }
    else { FILL }
}

pub fn fill_right_excl(m: u64) -> u64 {
    if m > 0 { shr_unchecked(FILL, (m.leading_zeros() + 1).into()) }
    else { FILL }
}

// Fills byte containg bit number i
pub fn byte_mask(i: usize) -> u64 {
    0xff << (i & 0b111000)
}

pub fn _print_bitboard(b: u64) {
    for i in (0..64).rev() {
        let b = (b >> i) & 1;
        let s = if b == 0 { '.' } else { 'x' };
        print!("{} ", s);
        if i % 8 == 0 {
            println!("");
        }
    }
    println!("");
}

pub struct BitIterator {
    value: u64,
    offset: u32,
}

impl BitIterator {
    pub fn new(value: u64) -> Self { Self { value, offset: 0 } }
}

impl Iterator for BitIterator {

    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.value == 0 { None } else {
            self.offset = self.value.trailing_zeros();
            let bit = 1 << self.offset;
            self.value = self.value & !bit;
            Some(bit)
        }        
    }
}

#[cfg(test)]
mod test {

    use crate::utils::*;

    #[test]
    fn bit_iterator() {
        let mut it   = BitIterator::new(0b101110);
        assert_eq!(it.next(), Some(0b000010));
        assert_eq!(it.next(), Some(0b000100));
        assert_eq!(it.next(), Some(0b001000));
        assert_eq!(it.next(), Some(0b100000));
        assert_eq!(it.next(), None);
    }
    
    #[test]
    fn fill() {
        let x = 0x00_00_00_00_08_00_00_00;
        assert_eq!(fill_left_incl(x), 0xff_ff_ff_ff_f8_00_00_00);
        assert_eq!(fill_left_excl(x), 0xff_ff_ff_ff_f0_00_00_00);
        assert_eq!(fill_right_incl(x), 0x00_00_00_00_0f_ff_ff_ff);
        assert_eq!(fill_right_excl(x), 0x00_00_00_00_07_ff_ff_ff);
    }

    #[test]
    fn flatten() {
        let x = 2;
        let y = 1;
        assert_eq!(flatten_bit(x, y), 0b100_00000000);
    }

    #[test]
    fn unflatten() {
        let b = 0b100_00000000; 
        assert_eq!(unflatten_bit(b), (2, 1));
    }

    #[test]
    fn bytemask() {
        let i = 10;
        assert_eq!(byte_mask(i), 0xff00);
    }
}
