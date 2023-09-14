

use crate::piece::Piece;
use crate::player::Player;
use crate::moves::MOVES;
use crate::utils;

const PIECE_COUNT: usize = 16;

mod index {

    use super::Piece; 

    pub const KING:    usize = 0;
    pub const KNIGHT: [usize; 2] = [1, 2];
    pub const ROOK:   [usize; 2] = [3, 4];
    pub const QUEEN:   usize = 5;
    pub const BISHOP: [usize; 2] = [6, 7];
    pub const PAWN:   [usize; 8] = [8, 9, 10, 11, 12, 13, 14, 15];

    pub fn into_piece(id: usize) -> Piece {
        match id {
            0     => Piece::King,
            1 | 2 => Piece::Knight,
            3 | 4 => Piece::Rook,
            5     => Piece::Queen,
            6 | 7 => Piece::Bishop,
            8..   => Piece::Pawn,
            _     => panic!(),
        }
    }
}


#[derive(Clone, Copy, Default)]
struct Team {
    // bitboard with one bit set, corresponding to piece's position
    // if ==0, piece is captured
    positions: [u64; PIECE_COUNT],
    // if > 0, locks correspnding piece to line/rank/diagonal
    pins: [u64; PIECE_COUNT],
    // positions of *opponent* pieces attacking king
    // if > 0, king is in check
    attacks: u64,
    // rays from *current* pieces attacking king ( may be blocked )
    sliding_attacks: [u64; PIECE_COUNT],
}

impl Team {

    pub fn pos_mask(&self) -> u64 {
        let mut m = 0;
        for p in self.positions.iter() {
            m |= p;
        }
        m
    }

    #[allow(dead_code)]
    pub fn ortho_mask(&self) -> u64 {
        self.positions[index::ROOK[0]] | 
        self.positions[index::ROOK[1]] | 
        self.positions[index::QUEEN]
    }

    #[allow(dead_code)]
    pub fn diag_mask(&self) -> u64 {
        self.positions[index::QUEEN] |
        self.positions[index::BISHOP[0]] | 
        self.positions[index::BISHOP[1]]
    }

    pub fn knight_mask(&self) -> u64 {
        self.positions[index::KNIGHT[0]] | 
        self.positions[index::KNIGHT[1]]
    }

    pub fn pawn_mask(&self) -> u64 {
        let mut m = 0;
        for p in &self.positions[index::PAWN[0]..] {
            m |= p;
        }
        m
    }
}

#[derive(Default)]
pub struct Board {
    current: Team,
    opponent: Team,
    pub player: Player,
}

impl Board {

    pub fn new() -> Board {

        use { index::*, utils::*, };
        let mut b = Board { player: Player::White, ..Default::default() };

        b.current.positions[ROOK[0]]   = flatten_bit(0, 0);
        b.current.positions[KNIGHT[0]] = flatten_bit(1, 0);
        b.current.positions[BISHOP[0]] = flatten_bit(2, 0);
        b.current.positions[QUEEN]     = flatten_bit(3, 0);
        b.current.positions[KING]      = flatten_bit(4, 0);
        b.current.positions[BISHOP[1]] = flatten_bit(5, 0);
        b.current.positions[KNIGHT[1]] = flatten_bit(6, 0);
        b.current.positions[ROOK[1]]   = flatten_bit(7, 0);
        
        for i in 0..8 {
            b.current.positions[PAWN[i]] = flatten_bit(i as u8, 1);
        }

        b.opponent.positions[ROOK[0]]   = flatten_bit(0, 7);
        b.opponent.positions[KNIGHT[0]] = flatten_bit(1, 7);
        b.opponent.positions[BISHOP[0]] = flatten_bit(2, 7);
        b.opponent.positions[QUEEN]     = flatten_bit(3, 7);
        b.opponent.positions[KING]      = flatten_bit(4, 7);
        b.opponent.positions[BISHOP[1]] = flatten_bit(5, 7);
        b.opponent.positions[KNIGHT[1]] = flatten_bit(6, 7);
        b.opponent.positions[ROOK[1]]   = flatten_bit(7, 7);
        
        for i in 0..8 {
            b.opponent.positions[PAWN[i]] = flatten_bit(i as u8, 6);
        }

        b
    }

    pub fn black_iter(&self) -> TeamIterator {
        match self.player {
            Player::White => TeamIterator::new(&self.opponent),
            Player::Black => TeamIterator::new(&self.current),
        }
    }

    pub fn white_iter(&self) -> TeamIterator {
        match self.player {
            Player::White => TeamIterator::new(&self.current),
            Player::Black => TeamIterator::new(&self.opponent),
        }
    }

    pub fn switch(&mut self) { 
        (self.current, self.opponent) = (self.opponent, self.current);
        self.player = match self.player {
            Player::White => Player::Black,
            Player::Black => Player::White,
        };
    }

    pub fn get_legal_moves(&self, id: usize) -> u64 {

        let piece = index::into_piece(id);

        let mov = match piece {
            Piece::Pawn   => self.psuedo_legal_pawn(id),
            Piece::King   => self.pseudo_legal_king(),
            Piece::Knight => self.pseudo_legal_knight(id),
            Piece::Bishop => self.pseudo_legal_diag(id),
            Piece::Rook   => self.pseudo_legal_ortho(id),
            Piece::Queen  => self.pseudo_legal_diag(id)
                           | self.pseudo_legal_ortho(id),
        };
        
        match piece {
            Piece::King => self.restrict_king_moves(mov),
            _ => self.restrict_moves(mov, id),
        }
    }

    pub fn id_from_pos(&self, x: u8, y: u8) -> Option<usize> {
        let pos1 = utils::flatten_bit(x, y);
        for (id, pos2) in self.current.positions.iter().enumerate() {
            if pos1 & pos2 > 0 {
                return Some(id);
            }
        }
        None
    }

    pub fn play_move(&mut self, piece_id: usize, dest: u64) {
        // move piece 
        self.current.positions[piece_id] = dest;
        // check for capture
        for c in &mut self.opponent.positions {
            if *c == dest {
                *c = 0;
                break;
            }
        }
        self.switch();
        self.comp_attacks();
        self.comp_pins();
    }

    fn pseudo_legal_king(&self) -> u64 {

        let position = self.current.positions[index::KING];
        let mut moves = MOVES.king_moves[position.trailing_zeros() as usize];
        moves &= !self.current.pos_mask();
        moves
    }

    fn pseudo_legal_knight(&self, id: usize) -> u64 {

        let position = self.current.positions[id];
        let mut moves = MOVES.knight_moves[position.trailing_zeros() as usize];
        moves &= !self.current.pos_mask();
        moves
    }
    
    fn psuedo_legal_pawn(&self, id: usize) -> u64 {

        let position = self.current.positions[id];
        let pos_tup = utils::unflatten_bit(position);
        let opp_mask = self.opponent.pos_mask();
        let curr_mask = self.current.pos_mask();
        let mut moves = 0;

        let mut single = MOVES.pawn_moves[utils::flatten(pos_tup.0, pos_tup.1)];
        let mut attack = MOVES.pawn_attacks[utils::flatten(pos_tup.0, pos_tup.1)];

        single &= !opp_mask;
        single &= !curr_mask;
        moves |= single;

        let first;
        let mut double;
        match self.player {
            Player::White => {
                first = pos_tup.1 == 1;
                double = single << 8;
                single &= utils::fill_left_excl(position);
                attack &= utils::fill_left_excl(position);
            },
            Player::Black => {
                first = pos_tup.1 == 6;
                double = single >> 8;
                single &= utils::fill_right_excl(position);
                attack &= utils::fill_right_excl(position);
            },
        };

        // Can only move to free squares
        single &= !curr_mask;
        single &= !opp_mask;
        
        // Can only move double if first row and single move
        if single > 0 && first {
            double &= !opp_mask;
            double &= !curr_mask;
            moves |= double;
        };

        // Captures available it opponent is there
        attack &= opp_mask;
        moves |= attack;

        moves
    }

    fn pseudo_legal_ortho(&self, id: usize) -> u64 {

        let pos_id    = self.current.positions[id].trailing_zeros() as usize;
        let mut moves = 0;
        let opp_mask  = self.opponent.pos_mask();
        let curr_mask = self.current.pos_mask();
        
        // north
        let opp_block   = utils::fill_right_incl(MOVES.north[pos_id] & opp_mask);
        let curr_block  = utils::fill_right_excl(MOVES.north[pos_id] & curr_mask);
        moves          |= MOVES.north[pos_id] & opp_block & curr_block;

        // west
        let opp_block   = utils::fill_right_incl(MOVES.west[pos_id] & opp_mask);
        let curr_block  = utils::fill_right_excl(MOVES.west[pos_id] & curr_mask);
        moves          |= MOVES.west[pos_id] & opp_block & curr_block;
        
        // south
        let opp_block   = utils::fill_left_incl(MOVES.south[pos_id] & opp_mask);
        let curr_block  = utils::fill_left_excl(MOVES.south[pos_id] & curr_mask);
        moves          |= MOVES.south[pos_id] & opp_block & curr_block;

        // east
        let opp_block   = utils::fill_left_incl(MOVES.east[pos_id] & opp_mask);
        let curr_block  = utils::fill_left_excl(MOVES.east[pos_id] & curr_mask);
        moves          |= MOVES.east[pos_id] & opp_block & curr_block;
        
        moves
    }

    fn pseudo_legal_diag(&self, id: usize) -> u64 {

        let pos_id    = self.current.positions[id].trailing_zeros() as usize;
        let mut moves = 0;
        let opp_mask  = self.opponent.pos_mask();
        let curr_mask = self.current.pos_mask();

        // north_east
        let opp_block   = utils::fill_right_incl(MOVES.north_east[pos_id] & opp_mask);
        let curr_block  = utils::fill_right_excl(MOVES.north_east[pos_id] & curr_mask);
        moves          |= MOVES.north_east[pos_id] & opp_block & curr_block;

        // north_west
        let opp_block   = utils::fill_right_incl(MOVES.north_west[pos_id] & opp_mask);
        let curr_block  = utils::fill_right_excl(MOVES.north_west[pos_id] & curr_mask);
        moves          |= MOVES.north_west[pos_id] & opp_block & curr_block;
        
        // south_east
        let opp_block   = utils::fill_left_incl(MOVES.south_east[pos_id] & opp_mask);
        let curr_block  = utils::fill_left_excl(MOVES.south_east[pos_id] & curr_mask);
        moves          |= MOVES.south_east[pos_id] & opp_block & curr_block;

        // south_west
        let opp_block   = utils::fill_left_incl(MOVES.south_west[pos_id] & opp_mask);
        let curr_block  = utils::fill_left_excl(MOVES.south_west[pos_id] & curr_mask);
        moves          |= MOVES.south_west[pos_id] & opp_block & curr_block;
        
        moves
    }

    // computes attacks from opponent pieces to king
    fn comp_attacks(&mut self) {

        // King position as bitboard
        let king_pos = self.current.positions[index::KING];
        // King position as square index
        let king_pos_id: usize = king_pos.trailing_zeros().try_into().unwrap();
        // Reset attacks
        self.current.attacks = 0;
        
        let mut pawn = MOVES.pawn_attacks[king_pos_id];
        // Pawn only moves forward, mask moves
        pawn &= match self.player {
            Player::White => utils::fill_left_excl(king_pos),
            Player::Black => utils::fill_right_excl(king_pos),
        };

        // Check overlap with opponents pawn
        self.current.attacks |= pawn & self.opponent.pawn_mask();

        let knight = MOVES.knight_moves[king_pos_id];
        self.current.attacks |= knight & self.opponent.knight_mask();
        
        let curr_mask = self.current.pos_mask();
        let opp_mask = self.opponent.pos_mask();

        for i in index::ROOK[0]..=index::QUEEN {
            let b = self.opponent.positions[i];
            // piece may be captured
            if b == 0 { continue; }
            let r = utils::ortho_ray_between_excl(b, king_pos);
            // add ray to sliding attacks, to compute pins
            // include attacking piece in pin
            self.opponent.sliding_attacks[i] = r | b;
            // is there a ray between them?
            let incident = r > 0;
            // is a piece (that isn't attacking piece) blocking?
            let blocked = r & (curr_mask | (opp_mask & !b)) > 0;
            self.current.attacks |= b * (incident && !blocked) as u64;
        }

        for i in index::QUEEN..=index::BISHOP[1] {
            let b = self.opponent.positions[i];
            if b == 0 { continue; }
            let r = utils::diag_ray_between_excl(b, king_pos);
            self.opponent.sliding_attacks[i] = r | b;
            let incident = r > 0;
            let blocked = r & (curr_mask | (opp_mask & !b)) > 0;
            self.current.attacks |= b * (incident && !blocked) as u64;
        }
    }

    fn comp_pins(&mut self) {

        let curr_mask = self.current.pos_mask();
        self.current.pins = [0; PIECE_COUNT];

        for r in self.opponent.sliding_attacks {
            // check for intersections with current pieces
            // and ray from sliding opponent piece that may attack king
            let intersections = (curr_mask & r).count_ones();
            // if only one piece blocks, it is pinned
            if intersections == 1 {
                for i in 0..PIECE_COUNT {
                    // here we find the piece that is pinned
                    if self.current.positions[i] & r > 0 {
                        self.current.pins[i] |= r;
                        break;
                    }
                }
            }
        }
    }

    // takes opponent attacks and pins in to account
    fn restrict_moves(&self, mut moves: u64, id: usize) -> u64 {

        let pins = self.current.pins[id];
        if pins > 0 {
            moves &= pins;
        }

        let attacks = self.current.attacks;
        match attacks.count_ones() {
            0 => (), // no restriction because no pieces are attacking
            1 => moves &= attacks, // only viable move is capture attacking piece
            _ => moves = 0, // more than one attacking -> king must move and no one else
        };

        moves
    }

    fn restrict_king_moves(&self, mut moves: u64) -> u64 {
        
        // we'll be looking for blocking pieces, so exclude king
        let curr_mask = self.current.pos_mask() 
            & !self.current.positions[index::KING];

        'iter_moves: for mov in utils::BitIterator::new(moves) {

            let mov_id: usize = mov.trailing_zeros().try_into().unwrap();

            // Pawns
            let mut pawns = MOVES.pawn_attacks[mov_id];
            // pawn only moves forward
            pawns &= match self.player {
                Player::White => utils::fill_left_excl(mov),
                Player::Black => utils::fill_right_excl(mov),
            };

            if pawns & self.opponent.pawn_mask() > 0 {
                moves &= !mov;
                continue 'iter_moves;
            }

            // Knights
            let knights = MOVES.knight_moves[mov_id];
            if knights & self.opponent.knight_mask() > 0 {
                moves &= !mov;
                continue 'iter_moves;
            }

            // Orthogonal sliding
            for opp_id in index::ROOK[0]..=index::QUEEN {

                let opp_pos = self.opponent.positions[opp_id];
                // Piece may be captured
                if opp_pos == 0 { continue; }
                let ray = utils::ortho_ray_between_excl(opp_pos, mov);
                if ray > 0 && ray & curr_mask == 0 {
                    moves &= !mov;
                    continue 'iter_moves;
                }
            }

            // Diagonal sliding
            for opp_id in index::QUEEN..=index::BISHOP[1] {

                let opp_pos = self.opponent.positions[opp_id];
                // Piece may be captured
                if opp_pos == 0 { continue; }
                let ray = utils::diag_ray_between_excl(opp_pos, mov);
                if ray > 0 && ray & curr_mask == 0 {
                    moves &= !mov;
                    continue 'iter_moves;
                }
            }

        }

        moves
    }

    pub fn is_checkmate(&self) -> bool {

        let mut king_moves = self.pseudo_legal_king();
        king_moves = self.restrict_king_moves(king_moves);
        let mut attacking = self.current.attacks.count_ones();

        if attacking == 1 {
            for id in 0..PIECE_COUNT {
                if self.is_attacking(id, self.current.attacks) {
                    attacking = 0;
                    break;
                }
            }
        }
        
        // if king can't move and there are multiple attacking pieces,
        // or the one attacking piece can't be captured
        // checkmate
        king_moves == 0 && attacking > 0
    }

    // Can current piece reach target?
    // false if piece is captured
    fn is_attacking(&self, att_id: usize, target: u64) -> bool {

        let pos = self.current.positions[att_id];
        // piece may be captured
        if pos == 0 { return false; }
        
        match index::into_piece(att_id) {
            Piece::Pawn => {
                let mut movs = MOVES.pawn_attacks[att_id];
                match self.player {
                    Player::White => movs &= utils::fill_left_excl(pos),
                    Player::Black => movs &= utils::fill_right_excl(pos),
                }
                movs & target > 0
            },
            Piece::Knight => {
                let movs = MOVES.knight_moves[att_id];
                movs & target > 0
            },
            _ => {

                let curr_mask = self.current.pos_mask();
                let opp_mask = self.opponent.pos_mask();

                if att_id >= index::ROOK[0] && att_id <= index::QUEEN {
                    let ray = utils::ortho_ray_between_excl(pos, target); 
                    if ray > 0 && ray & (curr_mask | opp_mask) == 0 {
                        return true;
                    }
                }

                if att_id >= index::QUEEN && att_id <= index::BISHOP[1] {
                    let ray = utils::diag_ray_between_excl(pos, target); 
                    if ray > 0 && ray & (curr_mask | opp_mask) == 0 {
                        return true;
                    }
                }

                false
            },
        }
    }
}

pub struct TeamIterator<'a> {
    team: &'a Team,
    id: usize,
}

impl<'a> TeamIterator<'a> {

    fn new(team: &'a Team) -> TeamIterator<'a> {
        TeamIterator {
            team,
            id: 0,
        }
    }
}

impl<'a> Iterator for TeamIterator<'a> {
    
    type Item = (Piece, u8, u8);

    fn next(&mut self) -> Option<(Piece, u8, u8)> {
        if self.id < 16 {
            let mut bit = self.team.positions[self.id];
            // piece may be captured
            while bit == 0 {
                self.id += 1;
                if self.id == 16 {
                    return None;
                }
                bit = self.team.positions[self.id];
            }
            let pos = utils::unflatten_bit(bit);
            let piece = index::into_piece(self.id);
            Some((piece, pos.0, pos.1)) 
        } else { None }
    }
}

#[cfg(test)]
mod test {

    use crate::{
        board::*,
        utils,
        player::*,
        piece::*,
    };

    #[test]
    fn movements() {
        
        let mut board = Board::new();

        // MOVE WHITE PAWN
        let id = board.id_from_pos(1, 1).unwrap();

        // Is pawn?
        assert!(matches!(index::into_piece(id), Piece::Pawn));
        let moves = board.get_legal_moves(id);
        let mov = utils::flatten_bit(1, 2);

        // Move exists?
        assert!(moves & mov > 0);
        board.play_move(id, mov);

        // Player switched?
        assert!(matches!(board.player, Player::Black));

        // Move happened?
        assert!(board.opponent.positions[id] & mov > 0);
        assert!(board.opponent.pos_mask() & utils::flatten_bit(1, 1) == 0);

        // MOVE BLACK PAWN DOUBLE
        let id = board.id_from_pos(0, 6).unwrap();
        let moves = board.get_legal_moves(id);
        let mov = utils::flatten_bit(0, 4);

        // Move exists?
        assert!(moves & mov > 0);
        board.play_move(id, mov);

        // MOVE WHITE BISHOP
        let id = board.id_from_pos(2, 0).unwrap();
        let moves = board.get_legal_moves(id);
        let mov = utils::flatten_bit(1, 1);

        // Move exists?
        assert!(moves & mov > 0);
        board.play_move(id, mov);

        // MOVE BLACK PAWN
        let id = board.id_from_pos(0, 4).unwrap();
        let moves = board.get_legal_moves(id);
        let mov = utils::flatten_bit(0, 3);

        // Move exists?
        assert!(moves & mov > 0);
        board.play_move(id, mov);

        // CAPTURE BLACK PAWN WITH WHITE BISHOP
        let id = board.id_from_pos(1, 1).unwrap();
        let moves = board.get_legal_moves(id);
        let mov = utils::flatten_bit(6, 6);

        // Move exists?
        assert!(moves & mov > 0);
        board.play_move(id, mov);

        // Capture happened?
        assert!(board.current.pos_mask() & mov == 0); 
        assert!(board.opponent.pos_mask() & mov > 0);

        // Not in checkmate
        assert!(!board.is_checkmate());
    }
}
