

use crate::piece::Piece;
use crate::player::Player;
use crate::moves::MOVES;
use crate::utils;

mod index {

    use super::Piece; 

    pub const KING:   usize = 0;
    pub const QUEEN:  usize = 1;
    pub const ROOK:   [usize; 2] = [2, 3];
    pub const KNIGHT: [usize; 2] = [4, 5];
    pub const BISHOP: [usize; 2] = [6, 7];
    pub const PAWN:   [usize; 8] = [8, 9, 10, 11, 12, 13, 14, 15];

    pub fn into_piece(id: usize) -> Piece {
        match id {
            0     => Piece::King,
            1     => Piece::Queen,
            2 | 3 => Piece::Rook,
            4 | 5 => Piece::Knight,
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
    positions: [u64; 16],
    // pins: [u64; 16]
}

impl Team {

    pub fn mask(&self) -> u64 {
        let mut m = 0;
        for p in self.positions.iter() {
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

    pub fn get_pseudo(&self, id: usize) -> u64 {
        match index::into_piece(id) {
            Piece::Pawn   => self.psuedo_legal_pawn(id, self.player),
            Piece::King   => self.pseudo_legal_king(),
            Piece::Knight => self.pseudo_legal_knight(id),
            Piece::Bishop => self.pseudo_legal_diag(id),
            Piece::Rook   => self.pseudo_legal_ortho(id),
            Piece::Queen  => self.pseudo_legal_diag(id)
                            | self.pseudo_legal_ortho(id),
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
    }

    fn pseudo_legal_king(&self) -> u64 {

        // TODO: avoid check

        let position = self.current.positions[index::KING];
        let mut moves = MOVES.king_moves[position.trailing_zeros() as usize];
        moves &= !self.current.mask();
        moves
    }

    fn pseudo_legal_knight(&self, id: usize) -> u64 {

        let position = self.current.positions[id];
        let mut moves = MOVES.knight_moves[position.trailing_zeros() as usize];
        moves &= !self.current.mask();
        moves
    }
    
    fn psuedo_legal_pawn(&self, id: usize, player: Player) -> u64 {

        // TODO: En passant
        
        let position = self.current.positions[id];
        let pos_tup = utils::unflatten_bit(position);
        let opp_mask = self.opponent.mask();
        let curr_mask = self.current.mask();
        let mut moves = 0;

        let (
            mut single, mut double,
            mut attack_r, mut attack_l,
            first,
        ) = match player {
            Player::White => (
                position << 8, // single
                position << 16, // double
                position << 7, // attack_r
                position << 9, // attack_l
                pos_tup.1 == 1,
            ),
            Player::Black => (
                position >> 8, // single
                position >> 16, // double
                position >> 9, // attack_r
                position >> 7, // attack_l
                pos_tup.1 == 6,
            ),
        };

        // Prevent wrapping
        if pos_tup.0 == 0 { attack_r = 0; }
        if pos_tup.0 == 7 { attack_l = 0; }
        
        // Can't move forward on opponent or current
        single &= !opp_mask;
        single &= !curr_mask;
        moves |= single;
        
        // Can only move double if first row and single move
        if single > 0 && first {
            // TODO: Register double move for en passant
            double &= !opp_mask;
            double &= !curr_mask;
            moves |= double;
        };

        // Captures available it opponent is there
        attack_r &= opp_mask;
        attack_l &= opp_mask;
        moves |= attack_r | attack_l;

        moves
    }

    fn pseudo_legal_ortho(&self, id: usize) -> u64 {

        let pos_id    = self.current.positions[id].trailing_zeros() as usize;
        let mut moves = 0;
        let opp_mask  = self.opponent.mask();
        let curr_mask = self.current.mask();
        
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
        let opp_mask  = self.opponent.mask();
        let curr_mask = self.current.mask();

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
                if (self.id == 16) {
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
        let moves = board.get_pseudo(id);
        let mov = utils::flatten_bit(1, 2);

        // Move exists?
        assert!(moves & mov > 0);
        board.play_move(id, mov);

        // Player switched?
        assert!(matches!(board.player, Player::Black));

        // Move happened?
        assert!(board.opponent.positions[id] & mov > 0);
        assert!(board.opponent.mask() & utils::flatten_bit(1, 1) == 0);

        // MOVE BLACK PAWN DOUBLE
        let id = board.id_from_pos(0, 6).unwrap();
        let moves = board.get_pseudo(id);
        let mov = utils::flatten_bit(0, 4);

        // Move exists?
        assert!(moves & mov > 0);
        board.play_move(id, mov);

        // MOVE WHITE BISHOP
        let id = board.id_from_pos(2, 0).unwrap();
        let moves = board.get_pseudo(id);
        let mov = utils::flatten_bit(1, 1);

        // Move exists?
        assert!(moves & mov > 0);
        board.play_move(id, mov);

        // MOVE BLACK PAWN
        let id = board.id_from_pos(0, 4).unwrap();
        let moves = board.get_pseudo(id);
        let mov = utils::flatten_bit(0, 3);

        // Move exists?
        assert!(moves & mov > 0);
        board.play_move(id, mov);

        // CAPTURE BLACK PAWN WITH WHITE BISHOP
        let id = board.id_from_pos(1, 1).unwrap();
        let moves = board.get_pseudo(id);
        let mov = utils::flatten_bit(6, 6);

        // Move exists?
        assert!(moves & mov > 0);
        board.play_move(id, mov);

        // Capture happened?
        assert!(board.current.mask() & mov == 0); 
        assert!(board.opponent.mask() & mov > 0);
    }
}
