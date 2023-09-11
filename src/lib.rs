//! ## Backend for chess game
//!
//! The state of the [Game] struct is represented by one of the enum variants
//! * [State::SelectPiece],
//! * [State::SelectMove],
//! * [State::SelectPromotion] and
//! * [State::CheckMate].
//!
//! To change the current state, a function corresponding to the current state must be called.
//! For example, if game state is [State::SelectPiece], one can call [Game::select_piece] to
//! transition to [State::SelectMove]. Game state can be queried with [Game::get_state]. Calling
//! a method that does not correspond to the current state results in [Error::InvalidState].
//!
//! ### Example usage:
//! ```
//! let game = Game::new();
//! // ...
//! // In mouse click handler:
//! match game.get_state() {
//!     State::SelectPiece => {
//!         let position: Position = position_of_square_under_cursor();
//!         game.select_piece(position).unwrap();
//!     },
//!     State::SelectMove => {
//!         let position: Position = position_of_square_under_cursor();
//!         game.select_move(position).unwrap();
//!     },
//!     // etc...
//! }
//! ```

/// Represents a position on the board.
pub struct Position { pub x: isize, pub y: isize, }

/// The kind of [Piece].
#[derive(Clone, Copy)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

/// [PieceKind] but truncated to the available options when promoting.
pub enum Promotion {
    Rook,
    Knight,
    Bishop,
    Queen,
}

/// Respresents a chess piece.
#[derive(Clone, Copy)]
pub struct Piece {
    pub kind: PieceKind,
    // true if kind == Pawn and pawn has moved two steps in one move
    // true if kind == King or Rook if piece has moved
    predicate: bool,
}

impl Piece {
    fn from_promotion(promotion: Promotion) -> Self {
        Self {
            kind: match promotion {
                Promotion::Rook => PieceKind::Rook,
                Promotion::Knight => PieceKind::Knight,
                Promotion::Queen => PieceKind::Queen,
                Promotion::Bishop => PieceKind::Bishop,
            },
            predicate: false,
        }
    }
}

/// Represents a legal move, returned by [Game::get_moves].
#[derive(Clone, Copy)]
pub struct Move {
    piece: Piece,
    from: Position,
    to: Position,
    kind: MoveKind,
}

impl Move {
    /// Get the destination of this move.
    pub fn get_destination(&self) -> Position {
        self.to
    }
}

#[derive(Clone, Copy)]
enum MoveKind {
    Regular,
    Capture { other: Piece, },
}

/// Represents the current player
pub enum Player {
    White,
    Black,
}

struct BoardState {
    current: [[Piece; 8]; 8],
    opponent: [[Piece; 8]; 8],
}

/// Represents a game of chess.
pub struct Game {
    state: State,
    board: BoardState,
    player: Player,
}

/// Describing the state of the game.
#[derive(Clone, Copy)]
pub enum State {
    /// Current player must select a piece to move.
    SelectPiece,
    /// Current player must select a move to execute.
    SelectMove,
    /// Current player must select a piece to promote to.
    SelectPromotion,
    /// Current player is in check mate.
    CheckMate,
}

/// Error type for [Game] methods.
pub enum Error {
    /// The method was called while game was in the wrong state.
    InvalidState,
    /// The position provided was outside the bounds of the board.
    InvalidPosition,
}

impl Game {
    
    /// Creates a new game with pieces in inital positions.
    pub fn new() -> Game {
        unimplemented!()
    }

    /// Resets the game to its original state.
    pub fn reset(&mut self) {
        unimplemented!()
    }

    /// Returns the current state of the game.
    pub fn get_state(&self) -> State {
        self.state
    }

    /// Returns the current player.
    pub fn get_current_player(&self) -> Player {
        self.player
    }
    
    /// Select a piece to move.
    /// Returns [Error::InvalidState] if game state is not [State::SelectPiece].
    /// Returns [Error::InvalidPosition] if `position` is not on the board. 
    /// Does nothing if the position is empty or occupied by the opponent.
    pub fn select_piece(&mut self, position: Position) -> Result<(), Error> {
        unimplemented!()
    }

    /// Get a slice of legal moves for piece selected with [Game::select_piece].
    /// Returns [Error::InvalidState] if game state is not [State::SelectMove].
    pub fn get_moves(&self) -> Result<&[Move], Error> {
        unimplemented!()
    }

    /// Selects move corresponding to `position`.
    /// Returns [Error::InvalidState] if game state is not [State::SelectMove].
    /// Returns [Error::InvalidPosition] position is not on the board.
    /// If `position` doesn't correspond to a legal move, 
    /// the state is reverted back to [State::SelectPiece].
    pub fn select_move(&mut self, position: Position) -> Result<(), Error> {
        unimplemented!()
    }

    /// Selects a piece to promote to.
    /// Returns [Error::InvalidState] if game state is not [State::SelectPromotion].
    pub fn select_promotion(&mut self, promotion: Option<Promotion>) -> Result<(), Error> {
        unimplemented!()
    }

    /// Returns a slice of black pieces with corresponding positions.
    pub fn get_black_positions(&self) -> &[(PieceKind, Position)] {
        unimplemented!()
    }

    /// Returns a slice of white pieces with corresponding positions.
    pub fn get_white_positions(&self) -> &[(PieceKind, Position)] {
        unimplemented!()
    }
}
