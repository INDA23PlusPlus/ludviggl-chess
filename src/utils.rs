
const FILL: u64 = 0xffffffffffffffff;

pub fn flatten(x: u8, y: u8) -> usize {
    (x | (y << 3)) as usize
}

pub fn bit(b: u64) -> u64 {
    1 << b
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
pub fn fill_left_incl(m: u64) -> u64 {
    FILL << m.trailing_zeros()
}

pub fn fill_left_excl(m: u64) -> u64 {
    FILL.checked_shl(m.trailing_zeros() + 1).unwrap_or(0)
}

pub fn fill_right_incl(m: u64) -> u64 {
    FILL >> m.leading_zeros()
}

pub fn fill_right_excl(m: u64) -> u64 {
    FILL.checked_shr(m.leading_zeros() + 1).unwrap_or(0)
}

// Fills byte containg bit number i
pub fn byte_mask(i: usize) -> u64 {
    0xff << (i & 0b111000)
}

// Fills left of bitboard, including bit b
pub fn left_mask(b: u64) -> u64 {
    let m = 0xff & (0xff << (b & 0xff));
    m | (m << 8)  | (m << 16)
      | (m << 24) | (m << 32)
      | (m << 40) | (m << 48)
      | (m << 56)  
}

// Right and Excluding b
pub fn right_mask(b: u64) -> u64 {
    !left_mask(b) 
}

pub fn print_bitboard(b: u64) {
    for i in (0..64).rev() {
        let b = (b >> i) & 1;
        let s = if b == 0 { '.' } else { 'x' };
        print!("{} ", s);
        if i % 8 == 0 {
            println!("");
        }
    }
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
