
use crate::utils;

lazy_static! (
    pub static ref MOVES: Moves = Moves::init();
);

pub struct Moves {
    pub(crate) king_moves:   [u64; 64],
    pub(crate) knight_moves: [u64; 64],
    // pawn moves/attacks in both directions, must be masked
    // depending on player
    pub(crate) pawn_moves:   [u64; 64],
    pub(crate) pawn_attacks: [u64; 64],
    pub(crate) east:         [u64; 64],
    pub(crate) north_east:   [u64; 64],
    pub(crate) north:        [u64; 64],
    pub(crate) north_west:   [u64; 64],
    pub(crate) west:         [u64; 64],
    pub(crate) south_west:   [u64; 64],
    pub(crate) south:        [u64; 64],
    pub(crate) south_east:   [u64; 64],
    pub(crate) diag_pos:     [u64; 64],
    pub(crate) diag_neg:     [u64; 64],
}

const KING_KERNEL: &[(i8, i8)] = &[
    (-1, -1), (0, -1), (1, -1),
    (-1,  0), /*   */  (1,  0),
    (-1,  1), (0,  1), (1,  1),
];

const KNIGHT_KERNEL: &[(i8, i8)] = &[
    /*    */  (-1, -2), /*  */  (1, -2), /*    */
    (-2, -1), /*                      */ (2, -1),
    /*                                         */
    (-2,  1), /*                      */ (2,  1),
    /*    */  (-1,  2), /*  */  (1,  2), /*    */
];

const PAWN_MOVE_KERNEL: &[(i8, i8)] = &[
    (0, -1), (0, 1),
];

const PAWN_ATTACK_KERNEL: &[(i8, i8)] = &[
    (-1, -1), /*    */ (1, -1),
    /*                       */
    (-1,  1), /*    */ (1,  1),
];

fn is_valid(p: (i8, i8)) -> bool {
    p.0 >= 0 && p.0 < 8
        && p.1 >= 0 && p.1 < 8
}

fn restrict(p: (i8, i8)) -> Option<(u8, u8)> {
    if is_valid(p) {
        Some((p.0 as u8, p.1 as u8))
    } else { None }
}

fn offset(o: (u8, u8), p: (i8, i8)) -> (i8, i8) {
    (o.0 as i8 + p.0, o.1 as i8 + p.1)
}

impl Moves {

    pub fn init() -> Moves {
        let mut moves = Moves { 
            king_moves:   [0; 64],
            knight_moves: [0; 64],
            pawn_moves:   [0; 64],
            pawn_attacks: [0; 64],
            east:         [0; 64],
            north_east:   [0; 64],
            north:        [0; 64],
            north_west:   [0; 64],
            west:         [0; 64],
            south_west:   [0; 64],
            south:        [0; 64],
            south_east:   [0; 64],
            diag_pos:     [0; 64],
            diag_neg:     [0; 64],
        };

        // King and knight moves
        for i in 0..64 {
            let o = utils::unflatten(i);
            let mut m = 0; 
            for p in KING_KERNEL {
                match restrict(offset(o, *p)) {
                    None => (),
                    Some(p) => m |= utils::flatten_bit(p.0, p.1),
                }
            }
            moves.king_moves[i] = m;

            m = 0; 
            for p in KNIGHT_KERNEL {
                match restrict(offset(o, *p)) {
                    None => (),
                    Some(p) => m |= utils::flatten_bit(p.0, p.1),
                }
            }
            moves.knight_moves[i] = m;

            m = 0; 
            for p in PAWN_MOVE_KERNEL {
                match restrict(offset(o, *p)) {
                    None => (),
                    Some(p) => m |= utils::flatten_bit(p.0, p.1),
                }
            }
            moves.pawn_moves[i] = m;

            m = 0; 
            for p in PAWN_ATTACK_KERNEL {
                match restrict(offset(o, *p)) {
                    None => (),
                    Some(p) => m |= utils::flatten_bit(p.0, p.1),
                }
            }
            moves.pawn_attacks[i] = m;
        }

        // North
        let mut m = 0x0101010101010100;
        for i in 0..64 {
            moves.north[i] = m;
            m <<= 1;
        }

        // South
        m = 0x0080808080808080;
        for i in (0..64).rev() {
            moves.south[i] = m;
            m >>= 1;
        }

        // West
        m = 0xfe;
        for i in 0..64 {
            moves.west[i] = m & utils::byte_mask(i);
            m <<= 1;
        }

        // East
        m = 0x7f00000000000000;
        for i in (0..64).rev() {
            moves.east[i] = m & utils::byte_mask(i);
            m >>= 1;
        }

        // Diagonals
        let mut b = 1;
        for i in 0usize..64 {

            // Negative
            let m = utils::neg_diag_through(b);

            // Mask diagonals to get rays
            moves.diag_neg[i]   = m;
            moves.north_west[i] = m & utils::fill_left_excl(b);
            moves.south_east[i] = m & utils::fill_right_excl(b);

            // Positive
            let m = utils::pos_diag_through(b);

            moves.diag_pos[i]   = m;
            moves.north_east[i] = m & utils::fill_left_excl(b);
            moves.south_west[i] = m & utils::fill_right_excl(b);

            b <<= 1;
        }

        moves
    }
}
