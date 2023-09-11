//! ## Backend for chess game
//!
//! ### Example usage:
//! ```
//! match game.get_state() {
//!     State::SelectPiece => {
//!         let position: Position = let_player_pick_piece_to_move();
//!         game.select_piece(position).unwrap();
//!     },
//!     State::SelectMove => {
//!         for mov in game.get_moves().unwrap().iter() {
//!             show_player_this_option(mov);
//!         }
//!         // ...
//!         let position: Position = let_player_pick_move();
//!         game.select_move(position).unwrap();
//!     },
//!     // ...
//! }
//! ```

pub struct Position { x: isize, y: isize, }

#[derive(Clone, Copy)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

pub enum Promotion {
    Rook,
    Knight,
    Bishop,
    Queen,
}

pub struct Piece {
    kind: PieceKind,
    predicate: bool,
}

#[derive(Clone, Copy)]
pub struct Move {
    piece: Piece,
    from: Position,
    to: Position,
    kind: MoveKind,
}

#[derive(Clone, Copy)]
enum MoveKind {
    Regular,
    Capture { other: Piece, },
}

struct BoardState {
    current: [[Piece; 8]; 8],
    opponent: [[Piece; 8]; 8],
}

pub struct Game {
    state: State,
    board: BoardState,
}

#[derive(Clone, Copy)]
pub enum State {
    SelectPiece,
    SelectMove,
    SelectPromotion,
}

impl Game {
    
    /// Creates a new game with pieces in inital positions.
    pub fn new() -> Game {
        unimplemented!()
    }

    /// Get the current state of the game.
    pub fn get_state(&self) -> State {
        self.state
    }
    
    /// Select a piece to move.
    /// Returns error if game state is not [State::SelectPiece]
    pub fn select_piece(&mut self, position: Position) -> Result<(), ()> {
        unimplemented!()
    }

    /// Get a slice of legal moves for piece selected with [Game::select_piece].
    /// Returns an error if game state is not [State::SelectMove]
    pub fn get_moves(&self, position: Position) -> Result<&[Move], ()> {
        unimplemented!()
    }

    /// Selects move corresponding to `position`
    /// If move is not in the list of legal moves, simply reverts
    /// state back to [State::SelectPiece].
    /// Returns an error if game state is not [State::SelectMove]
    pub fn select_move(&mut self, position: Position) -> Result<(), ()> {
        unimplemented!()
    }

    /// Selects a piece to promote to.
    /// Returns an error if game state is not [State::SelectPromotion]
    pub fn select_promotion(&mut self, promotion: Option<Promotion>) -> Result<(), ()> {
        unimplemented!()
    }

    /// Returns a slice of black pieces with corresponding positions.
    pub fn get_black_positions(&self) -> &[(Piece, Position)] {
        unimplemented!()
    }

    /// Returns a slice of white pieces with corresponding positions.
    pub fn get_white_positions(&self) -> &[(Piece, Position)] {
        unimplemented!()
    }
}
