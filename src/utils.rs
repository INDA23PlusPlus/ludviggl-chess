
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

// Fills bits left of ms 1 of m, incl ms 1
pub fn fill_left_incl(m: u64) -> u64 {
    shl_unchecked(FILL, m.trailing_zeros().into())
}

pub fn fill_left_excl(m: u64) -> u64 {
    shl_unchecked(FILL, (m.trailing_zeros() + 1).into())
}

pub fn fill_right_incl(m: u64) -> u64 {
    shr_unchecked(FILL, m.leading_zeros().into())
}

pub fn fill_right_excl(m: u64) -> u64 {
    shr_unchecked(FILL, (m.leading_zeros() + 1).into())
}

// fill between bits b1 & b2, including b1 & b2
pub fn fill_between_incl(b1: u64, b2: u64) -> u64 {
    (fill_left_incl(b1) & fill_right_incl(b2)) |
    (fill_left_incl(b2) & fill_right_incl(b1))
}

pub fn fill_between_excl(b1: u64, b2: u64) -> u64 {
    (fill_left_excl(b1) & fill_right_excl(b2)) |
    (fill_left_excl(b2) & fill_right_excl(b1))
}

pub fn neg_diag_through(b: u64) -> u64 {

    debug_assert!(b > 0);

    const DN: u64 = 0x8040201008040201;
    let p = unflatten_bit(b);

    if p.0 >= p.1 {
        DN >> ((p.0 - p.1) << 3)
    } else {
        DN << ((p.1 - p.0) << 3)
    }
}

pub fn pos_diag_through(b: u64) -> u64 {

    debug_assert!(b > 0);

    const DP: u64 = 0x0102040810204080;
    let p = unflatten_bit(b);
    let z = 7 - p.0;

    if z >= p.1 {
        DP >> ((z - p.1) << 3)
    } else {
        DP << ((p.1 - z) << 3)
    }
}

// gets ray between bits, icluding endpoints
// returns 0 if not on same diagonal
pub fn diag_ray_between_incl(b1: u64, b2: u64) -> u64 {

    let dn = neg_diag_through(b1);
    let dp = pos_diag_through(b1);
    let d = (dn * (dn & b2 != 0) as u64) | 
        (dp * (dp & b2 != 0) as u64);
    d & fill_between_incl(b1, b2)
}

pub fn ortho_ray_between_incl(b1: u64, b2: u64) -> u64 {
    
    let h = byte_mask(b1.trailing_zeros().try_into().unwrap());
    let v = col_mask(b1.trailing_zeros().try_into().unwrap());
    let o = (h * (h & b2 != 0) as u64) |
        (v * (v & b2 != 0) as u64);
    o & fill_between_incl(b1, b2)
}

pub fn diag_ray_between_excl(b1: u64, b2: u64) -> u64 {

    let dn = neg_diag_through(b1);
    let dp = pos_diag_through(b1);
    let d = (dn * (dn & b2 != 0) as u64) | 
        (dp * (dp & b2 != 0) as u64);
    d & fill_between_excl(b1, b2)
}

pub fn ortho_ray_between_excl(b1: u64, b2: u64) -> u64 {
    
    let h = byte_mask(b1.trailing_zeros().try_into().unwrap());
    let v = col_mask(b1.trailing_zeros().try_into().unwrap());
    let o = (h * (h & b2 != 0) as u64) |
        (v * (v & b2 != 0) as u64);
    o & fill_between_excl(b1, b2)
}

// Fills byte containg bit number i
pub fn byte_mask(i: usize) -> u64 {
    0xff << (i & 0b111000)
}

pub fn col_mask(i: usize) -> u64 {
    0x0101010101010101 << (i & 0b111)
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
        let x = 0x00_00_00_00_0a_00_00_00;
        let v = fill_left_incl(x);
        let e = 0xff_ff_ff_ff_fe_00_00_00;
        println!("x: {:#066b}\nv: {:#066b}\ne: {:#066b}", x, v, e);
        assert_eq!(v, e);
        let v = fill_left_excl(x);
        let e = 0xff_ff_ff_ff_fc_00_00_00;
        println!("x: {:#066b}\nv: {:#066b}\ne: {:#066b}", x, v, e);
        assert_eq!(v, e);
        let v = fill_right_incl(x);
        let e = 0x00_00_00_00_0f_ff_ff_ff;
        println!("x: {:#066b}\nv: {:#066b}\ne: {:#066b}", x, v, e);
        assert_eq!(v, e);
        let v = fill_right_excl(x);
        let e = 0x00_00_00_00_07_ff_ff_ff;
        println!("x: {:#066b}\nv: {:#066b}\ne: {:#066b}", x, v, e);
        assert_eq!(v, e);
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

    #[test]
    fn fill_between_() {
        let b1 = 0b000010000;
        let b2 = 0b100000000;
        let e  = 0b111110000;
        assert_eq!(fill_between_incl(b1, b2), e);
    }

    #[test]
    fn ortho_ray_between_() {
        let b1 = 0x000002;
        let b2 = 0x020000;
        let e  = 0x020202;
        assert_eq!(ortho_ray_between_incl(b1, b2), e);
        let b2 = 0x040000;
        assert_eq!(ortho_ray_between_incl(b1, b2), 0);
    }
}
