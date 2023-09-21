
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


#[derive(Clone, Copy)]
struct Team {
    positions:      [u64; PIECE_COUNT],
    promotions:     [Option<Piece>; PIECE_COUNT],
    promotion_id:   isize,
    en_passant_pos: u64,
    did_castling:   bool,
    did_move:       u64,
    king_moved:     bool,
}

impl Team {
    
    fn mask(self: &Self) -> u64 {
        let mut m = 0;
        for &p in &self.positions[..] {
            m |= p; 
        }
        m
    }
}

impl Default for Team {

    fn default() -> Self {
        Self {
            positions:      [0; PIECE_COUNT],
            promotions:     [None; PIECE_COUNT],
            promotion_id:   -1,
            en_passant_pos: 0,
            did_castling:   false,
            did_move:       0,
            king_moved:      false,
        }
    }
}

#[derive(Default)]
pub struct Board {
    white: Team,
    black: Team,
    pub player: Player,
}

impl Board {

    pub fn new() -> Board {

        use { index::*, utils::*, };
        let mut b = Board { player: Player::White, ..Default::default() };

        b.white.positions[ROOK[0]]   = flatten_bit(0, 0);
        b.white.positions[KNIGHT[0]] = flatten_bit(1, 0);
        b.white.positions[BISHOP[0]] = flatten_bit(2, 0);
        b.white.positions[QUEEN]     = flatten_bit(3, 0);
        b.white.positions[KING]      = flatten_bit(4, 0);
        b.white.positions[BISHOP[1]] = flatten_bit(5, 0);
        b.white.positions[KNIGHT[1]] = flatten_bit(6, 0);
        b.white.positions[ROOK[1]]   = flatten_bit(7, 0);
        
        for i in 0..8 {
            b.white.positions[PAWN[i]] = flatten_bit(i as u8, 1);
        }

        b.black.positions[ROOK[0]]   = flatten_bit(0, 7);
        b.black.positions[KNIGHT[0]] = flatten_bit(1, 7);
        b.black.positions[BISHOP[0]] = flatten_bit(2, 7);
        b.black.positions[QUEEN]     = flatten_bit(3, 7);
        b.black.positions[KING]      = flatten_bit(4, 7);
        b.black.positions[BISHOP[1]] = flatten_bit(5, 7);
        b.black.positions[KNIGHT[1]] = flatten_bit(6, 7);
        b.black.positions[ROOK[1]]   = flatten_bit(7, 7);
        
        for i in 0..8 {
            b.black.positions[PAWN[i]] = flatten_bit(i as u8, 6);
        }

        b
    }

    pub fn white_iter(self: &Self) -> TeamIterator {
        TeamIterator::new(&self.white)
    }

    pub fn black_iter(self: &Self) -> TeamIterator {
        TeamIterator::new(&self.black)
    }

    pub fn has_promotion(self: &Self) -> bool { 
        (match self.player {
            Player::White => self.white.promotion_id,
            Player::Black => self.black.promotion_id,
        }) >= 0
    }

    pub fn is_checkmate(self: &Self) -> bool {
        
        // Just check if there are any available moves
        for id in 0..PIECE_COUNT {

            if (*match self.player {
                Player::White => &self.white,
                Player::Black => &self.black,
            }).positions[id] == 0 { continue; }

            if self.get_legal_moves(id) > 0 {
                return false;
            } 
        }

        true
    }

    pub fn select_promotion(self: &mut Self, piece: Piece) {

        let curr = match self.player {
            Player::White => &mut self.white,
            Player::Black => &mut self.black,
        };

        debug_assert!(curr.promotion_id >= 0);

        curr.promotions[curr.promotion_id as usize] 
            = Some(piece);
        curr.promotion_id = -1;
        
        use Player::*;
        self.player = match self.player {
            White => Black,
            Black => White,
        };
    }

    pub fn play_move(self: &mut Self, id: usize, mov: u64) {

        use Player::*;

        let (curr_team, opp_team) = match self.player {
            White => (&mut self.white, &mut self.black, ),
            Black => (&mut self.black, &mut self.white, ),
        };

        let mut att_pos = mov;

        // check en passant attack
        if id >= index::PAWN[0] && opp_team.en_passant_pos > 0 {

             let capt_pos = match self.player {
                 White => mov >> 8,
                 Black => mov << 8,
             };

             if opp_team.en_passant_pos == capt_pos {
                 att_pos = capt_pos;
             }
        }

        for p in &mut opp_team.positions[..] {
            if *p == att_pos {
                *p = 0;
                break;
            }
        }

        let pos = curr_team.positions[id];
        let mtz = mov.trailing_zeros() as i32;

        let dist = pos.trailing_zeros() as i32 - mtz;

        let mut switch = true;
        
        if id >= index::PAWN[0] {

            // update en passant pos
            let double_move = dist == 16 || dist == -16;

            if double_move {
                curr_team.en_passant_pos = mov;
            } else {
                curr_team.en_passant_pos = 0;
            }

            // check for promotion
            if mtz < 8 || mtz >= 56 {
                
                // Can't promote twice
                if matches!(curr_team.promotions[id], None) {
                    curr_team.promotion_id = id as isize;
                    switch = false;
                }
            }
        }

        // Castling
        match index::into_piece(id) {
            Piece::Rook => {
                curr_team.did_move |= mov;
            },
            Piece::King => {

                curr_team.king_moved = true;

                let (castl, cmask, roffset) = match dist {
                    -2 => ( // left
                        true,
                        utils::fill_left_excl(pos),
                        -1,
                    ),
                    2 => ( // right
                        true,
                        utils::fill_right_excl(pos),
                        1,
                    ),
                    _ => (false, 0, 0),
                };

                if castl {

                    curr_team.did_castling = true;

                    let rpos = match roffset {
                        -1 => mov >> 1,
                        1  => mov << 1,
                        _ => panic!(),
                    };

                    for rp in &mut curr_team.positions[index::ROOK[0]..=index::ROOK[1]] {
                        
                        if *rp & cmask > 0 {
                            *rp = rpos;
                        }
                    }
                }
            }
            _ => (),
        }

        curr_team.positions[id] = mov;

        if switch {
            self.player = match self.player {
                White => Black,
                Black => White,
            };
        }
    }

    pub fn get_legal_moves(self: &Self, id: usize) -> u64 {
        
        let (curr_team, opp_team) = match self.player {
            Player::White => (&self.white, &self.black, ),
            Player::Black => (&self.black, &self.white, ),
        };

        let pos = curr_team.positions[id];

        use Piece::*;
        let curr = curr_team.mask();
        let opp = opp_team.mask();
        let mut moves = match index::into_piece(id) {
            // With pawn we must check if it has been promoted first
            Pawn   => if let Some(piece) = curr_team.promotions[id] {
                    // Promotion
                    match piece {
                        Rook   => Self::ortho_unrestr(pos, curr, opp),
                        Bishop => Self::diag_unrestr(pos, curr, opp),
                        Queen  => Self::ortho_unrestr(pos, curr, opp)
                                | Self::diag_unrestr(pos, curr, opp),
                        _      => panic!(),
                    }
                } else {
                    // Regular pawn :/
                    Self::pawn_unrestr(
                        pos,
                        curr,
                        opp,
                        self.player,
                        opp_team.en_passant_pos
                    )
            },
            Knight => Self::knight_unrestr(pos, curr, opp),
            King   => Self::king_unrestr(pos, curr, opp),
            Bishop => Self::diag_unrestr(pos, curr, opp),
            Rook   => Self::ortho_unrestr(pos, curr, opp),
            Queen  => Self::diag_unrestr(pos, curr, opp)
                    | Self::ortho_unrestr(pos, curr, opp),
        };

        if id == index::KING {

            moves = Self::restrict_king(
                moves,
                curr,
                opp,
                &opp_team.positions,
                &opp_team.promotions,
                self.player
            );

            moves |= Self::castling_moves(pos, &curr_team, &opp_team, self.player);

        } else {

            let pins = Self::comp_pins(
                pos,
                curr,
                opp,
                &opp_team.positions,
                &opp_team.promotions,
                curr_team.positions[index::KING],
                self.player
            );

            moves = Self::restrict(moves, pins);
        }

        moves
    }

    pub fn id_from_pos(self: &Self, x: u8, y: u8) -> Option<usize> {

        let b = utils::flatten_bit(x, y);
        let ps = &match self.player {
            Player::White => self.white.positions,
            Player::Black => self.black.positions,
        };

        for (id, &p) in ps.iter().enumerate() {
            if p == b { return Some(id); }
        }

        None
    }

    fn ortho_unrestr(pos: u64, curr: u64, opp: u64) -> u64 {

        debug_assert!(pos > 0); 

        let mut moves = 0;
        let i = pos.trailing_zeros() as usize;

        let mut m = MOVES.north[i];
        let cint = m & curr;
        let oint = m & opp;
        if cint + oint > 0 {
            let cblk = utils::fill_left_incl(cint);
            let oblk = utils::fill_left_excl(oint);
            m &= !(cblk | oblk);
        }
        moves |= m;

        let mut m = MOVES.west[i];
        let cint = m & curr;
        let oint = m & opp;
        if cint + oint > 0 {
            let cblk = utils::fill_left_incl(cint);
            let oblk = utils::fill_left_excl(oint);
            m &= !(cblk | oblk);
        }
        moves |= m;

        let mut m = MOVES.south[i];
        let cint = m & curr;
        let oint = m & opp;
        if cint + oint > 0 {
            let cblk = utils::fill_right_incl(cint);
            let oblk = utils::fill_right_excl(oint);
            m &= !(cblk | oblk);
        }
        moves |= m;

        let mut m = MOVES.east[i];
        let cint = m & curr;
        let oint = m & opp;
        if cint + oint > 0 {
            let cblk = utils::fill_right_incl(cint);
            let oblk = utils::fill_right_excl(oint);
            m &= !(cblk | oblk);
        }
        moves |= m;

        moves
    }

    fn diag_unrestr(pos: u64, curr: u64, opp: u64) -> u64 {

        debug_assert!(pos > 0); 

        let mut moves = 0;
        let i = pos.trailing_zeros() as usize;

        let mut m = MOVES.north_east[i];
        let cint = m & curr;
        let oint = m & opp;
        if cint + oint > 0 {
            let cblk = utils::fill_left_incl(cint);
            let oblk = utils::fill_left_excl(oint);
            m &= !(cblk | oblk);
        }
        moves |= m;

        let mut m = MOVES.north_west[i];
        let cint = m & curr;
        let oint = m & opp;
        if cint + oint > 0 {
            let cblk = utils::fill_left_incl(cint);
            let oblk = utils::fill_left_excl(oint);
            m &= !(cblk | oblk);
        }
        moves |= m;

        let mut m = MOVES.south_west[i];
        let cint = m & curr;
        let oint = m & opp;
        if cint + oint > 0 {
            let cblk = utils::fill_right_incl(cint);
            let oblk = utils::fill_right_excl(oint);
            m &= !(cblk | oblk);
        }
        moves |= m;

        let mut m = MOVES.south_east[i];
        let cint = m & curr;
        
        let oint = m & opp;
        if cint + oint > 0 {
            let cblk = utils::fill_right_incl(cint);
            let oblk = utils::fill_right_excl(oint);
            m &= !(cblk | oblk);
        }
        moves |= m;

        moves
    }

    fn pawn_unrestr(
        pos: u64,
        curr: u64,
        opp: u64,
        player: Player,
        opp_en_passant: u64
    ) -> u64 {

        debug_assert!(pos > 0);

        let mut moves = 0;
        let i = pos.trailing_zeros() as usize;
        
        use Player::*;
        let msk = match player {
            White => utils::fill_left_excl(pos),
            Black => utils::fill_right_excl(pos),
        };

        moves |= MOVES.pawn_moves[i]
                    & msk   // Only forward
                    & !curr // Only empty squares
                    & !opp; // Including opponents

        if moves > 0 {
            let (double, first) = match player {
                White => (pos << 16, i >> 3 == 1),
                Black => (pos >> 16, i >> 3 == 6),
            };
            
            if first { // Only available as first move
                moves |= double 
                        & !curr 
                        & !opp;
            }
        }
        
        moves |= MOVES.pawn_attacks[i]
                    & msk   // Only forward
                    & opp;  // Only opponents

        // En passant
        moves |= match player {
            White => (((MOVES.pawn_attacks[i] & msk) >> 8)
                        & opp_en_passant) << 8,
            Black => (((MOVES.pawn_attacks[i] & msk) << 8)
                        & opp_en_passant) >> 8,
        } & !curr;

        moves
    }

    fn knight_unrestr(pos: u64, curr: u64, _opp: u64) -> u64 {

        debug_assert!(pos > 0);

        let i = pos.trailing_zeros() as usize;
        MOVES.knight_moves[i] & !curr
    }

    fn king_unrestr(pos: u64, curr: u64, _opp: u64) -> u64 {

        debug_assert!(pos > 0);

        let i = pos.trailing_zeros() as usize;
        MOVES.king_moves[i] & !curr
    }

    fn castling_moves(
        kpos: u64,
        curr_team: &Team,
        opp_team: &Team,
        player: Player
    ) -> u64 {

        let mut moves = 0;

        if curr_team.king_moved {
            return 0;
        }

        // Get available rooks
        let mut rooks = 0;
        for &p in &curr_team.positions[index::ROOK[0]..=index::ROOK[1]] {
            rooks |= p;
        }
        
        // Only rooks on the same row
        rooks &= utils::byte_mask(kpos.trailing_zeros() as usize);

        // Only rooks that haven't moved
        rooks &= !curr_team.did_move;

        let move_mask = kpos << 2 | kpos >> 2;

        for r in utils::BitIterator::new(rooks) {

            // Make sure positions are vacant
            let between = utils::ortho_ray_between_excl(r, kpos);
            if between & (curr_team.mask() | opp_team.mask()) > 0 {
                continue;
            }

            // Find move destination
            let mov = move_mask & between;

            // Make sure no square on the way is attacked
            for b in utils::BitIterator::new(utils::ortho_ray_between_excl(kpos, mov)) {

                if !Self::is_attacked(
                    b,
                    curr_team.mask(),
                    opp_team.mask(),
                    &opp_team.positions,
                    &opp_team.promotions,
                    player
                ) {
                    moves |= mov;
                }
            }
        }

        moves
    }

    fn ortho_can_reach(pos: u64, target: u64, blk: u64) -> bool {

        if pos == 0 { return false; }
        
        let ray = utils::ortho_ray_between_incl(pos, target);
    
        if ray == 0 || // no ray between points
            blk & (ray & !pos & !target) > 0 // ray is blocked
        {
            false
        } else {
            true
        }
    }

    fn diag_can_reach(pos: u64, target: u64, blk: u64) -> bool {

        if pos == 0 { return false; }
        
        let ray = utils::diag_ray_between_incl(pos, target);
    
        if ray == 0 || // no ray between points
            blk & (ray & !pos & !target) > 0 // ray is blocked
        {
            false
        } else {
            true
        }
    }

    fn restrict(mov: u64, pins: u64) -> u64 {
        mov & pins
    }

    fn restrict_king(
        moves: u64,
        curr: u64,
        opp: u64,
        opp_pos: &[u64],
        opp_prom: &[Option<Piece>],
        player: Player
    ) -> u64 {

        let mut moves = moves;
        
        for mov in utils::BitIterator::new(moves) {
            if Self::is_attacked(mov, curr, opp, opp_pos, opp_prom, player) {
                moves &= !mov;
            }    
        }

        moves
    }


    fn is_attacked(
        pos: u64,
        curr: u64,
        opp: u64,
        opp_pos: &[u64],
        opp_prom: &[Option<Piece>],
        player: Player
    ) -> bool {

        use { index::*, Player::*, };

        let id = pos.trailing_zeros() as usize;
        
        let pwn_att = MOVES.pawn_attacks[id]
            & match player {
                White => utils::fill_left_excl(pos),
                Black => utils::fill_right_excl(pos),
            };
        
        for i in PAWN[0]..=PAWN[7] {
            // May be promoted
            if !matches!(opp_prom[i], None) {
                continue;
            }
            let p = &opp_pos[i];
            if p & pwn_att > 0 {
                return true;
            }
        }
        
        let kn_poses = MOVES.knight_moves[id];
        if kn_poses & (opp_pos[KNIGHT[0]] | opp_pos[KNIGHT[1]]) > 0 {
            return true;
        }
        
        // Promoted pawns
        for i in PAWN[0]..=PAWN[7] {
            if let Some(Piece::Knight) = opp_prom[i] {
                let tz = opp_pos[i].trailing_zeros() as usize;
                let pkn_poses = MOVES.knight_moves[tz];
                if pkn_poses & pos > 0 {
                    return true;
                }
            }
        }

        for &p in &opp_pos[ROOK[0]..=QUEEN] {
            if Self::ortho_can_reach(p, pos, (curr & !pos) | opp) {
                if p == pos {
                    // We can capture it
                    continue;
                }
                return true;
            }
        }

        for &p in &opp_pos[QUEEN..=BISHOP[1]] {
            if Self::diag_can_reach(p, pos, (curr & !pos) | opp) {
                if p == pos {
                    // We can capture it
                    continue;
                }
                return true;
            }
        }
        
        // Promoted pawns
        for i in PAWN[0]..=PAWN[7] {
            if let Some(piece) = opp_prom[i] {

                let p = opp_pos[i];

                if matches!(piece, Piece::Rook) || matches!(piece, Piece::Queen) {

                    if Self::ortho_can_reach(p, pos, (curr & !pos) | opp) {
                        if p == pos {
                            // We can capture it
                            continue;
                        }
                        return true;
                    }
                }

                if matches!(piece, Piece::Bishop) || matches!(piece, Piece::Queen) {

                    if Self::diag_can_reach(p, pos, (curr & !pos) | opp) {
                        if p == pos {
                            // We can capture it
                            continue;
                        }
                        return true;
                    }
                }
            }
        }

        return false;
    }

    fn comp_pins(
        pos: u64,
        curr: u64,
        opp: u64,
        opp_pos: &[u64],
        opp_prom: &[Option<Piece>],
        king_pos: u64,
        player: Player
    ) -> u64 {

        let mut pins = !0u64;
        let king_id = king_pos.trailing_zeros() as usize;
        
        use { index::*, Player::*, };
        
        let pwn_att = MOVES.pawn_attacks[king_id] & match player {
            White => utils::fill_left_excl(king_pos),
            Black => utils::fill_right_excl(king_pos),
        };

        for i in PAWN[0]..=PAWN[7] {
            // May be promoted
            if !matches!(opp_prom[i], None) {
                continue;
            }
            let p = opp_pos[i];
            if pwn_att & p > 0 {
                pins &= p;
            }
        }

        let kn_mov = MOVES.knight_moves[king_id];

        for &p in &opp_pos[KNIGHT[0]..=KNIGHT[1]] {
            if kn_mov & p > 0 {
                pins &= p;
            }
        }

        for &o in &opp_pos[ROOK[0]..=QUEEN] {
            
            let ray = utils::ortho_ray_between_excl(king_pos, o);
            if ray == 0 {
                // It might be adjacent, in which case ray is empty
                // Thus we check inclusive ray
                if utils::ortho_ray_between_incl(king_pos, o) > 0 {
                    pins &= o;
                }
                continue;
            }
            let blockers = (ray & (curr | opp)).count_ones();
            if blockers == 0 || // Not blocked, must be blocked or captured
                blockers == 1 && ray & pos > 0 // Only blocker, must stay in lane or capture
            {
                pins &= ray | o;
            }
        }

        for &d in &opp_pos[QUEEN..=BISHOP[1]] {
            
            let ray = utils::diag_ray_between_excl(king_pos, d);
            if ray == 0 {
                // It might be adjacent, in which case ray is empty
                // Thus we check inclusive ray
                if utils::diag_ray_between_incl(king_pos, d) > 0 {
                    pins &= d;
                }
                continue;
            }
            let blockers = (ray & (curr | opp)).count_ones();
            if blockers == 0 || // Not blocked, must be blocked or captured
                blockers == 1 && ray & pos > 0 // Only blocker, must stay in lane or capture
            {
                pins &= ray | d;
            }
        }

        // Check promoted
        use Piece::*;
        for i in PAWN[0]..=PAWN[7] {
            if let Some(prom) = opp_prom[i] {
                let p = opp_pos[i];
                match prom {
                    Knight => {
                        if kn_mov & p > 0 {
                            pins &= p;
                        }
                    },
                    _ => {
                        if matches!(prom, Rook) || matches!(prom, Queen) {
                            
                            let ray = utils::ortho_ray_between_excl(king_pos, p);
                            if ray == 0 {
                                // It might be adjacent, in which case ray is empty
                                // Thus we check inclusive ray
                                if utils::ortho_ray_between_incl(king_pos, p) > 0 {
                                    pins &= p;
                                }
                                continue;
                            }
                            let blockers = (ray & (curr | opp)).count_ones();
                            if blockers == 0 || // Not blocked, must be blocked pr captured
                                blockers == 1 && ray & pos > 0 // Only blocker, must stay in lane pr capture
                            {
                                pins &= ray | p;
                            }
                        }

                        if matches!(prom, Bishop) || matches!(prom, Queen) {
                            
                            let ray = utils::diag_ray_between_excl(king_pos, p);
                            if ray == 0 {
                                // It might be adjacent, in which case ray is empty
                                // Thus we check inclusive ray
                                if utils::diag_ray_between_incl(king_pos, p) > 0 {
                                    pins &= p;
                                }
                                continue;
                            }
                            let blockers = (ray & (curr | opp)).count_ones();
                            if blockers == 0 || // Not blocked, must be blocked or captured
                                blockers == 1 && ray & pos > 0 // Only blocker, must stay in lane or capture
                            {
                                pins &= ray | p;
                            }
                        }
                    }
                }
            }
        }

        pins
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
                if self.id >= 16 {
                    return None;
                }
                bit = self.team.positions[self.id];
            }
            let pos = utils::unflatten_bit(bit);
            let piece = match self.team.promotions[self.id] {
                None => index::into_piece(self.id),
                Some(piece) => piece,
            };
            self.id += 1;
            Some((piece, pos.0, pos.1)) 
        } else { None }
    }
}
