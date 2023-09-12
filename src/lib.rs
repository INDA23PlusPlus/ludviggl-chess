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
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Position { pub x: isize, pub y: isize, }

impl Position {

    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y, }
    }

    fn is_valid(&self) -> bool {
        self.x >= 0 && self.x < 8
            && self.y >= 0 && self.y < 8
    }
}

impl std::ops::Add for Position {

    type Output = Self;
    
    fn add(self, rhs: Self) -> Self {
        Position::new(
            self.x + rhs.x,
            self.y + rhs.y,
        )
    }
}

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
    // Only used for rook/king to check for castling
    moved: bool,
}

impl Piece {

    fn new(kind: PieceKind) -> Self {

        Self {
            kind,
            moved: false,
        }
    }

    fn from_promotion(promotion: Promotion) -> Self {

        Self {
            kind: match promotion {
                Promotion::Rook => PieceKind::Rook,
                Promotion::Knight => PieceKind::Knight,
                Promotion::Queen => PieceKind::Queen,
                Promotion::Bishop => PieceKind::Bishop,
            },
            moved: false,
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
    /// Useful for displaying possible moves to player.
    pub fn get_destination(&self) -> Position {
        self.to
    }
}

#[derive(Clone, Copy)]
enum MoveKind {
    Regular,
    Double,
    Promotion,
    Castling { other_pos: Position },
    EnPassant,
}

/// Represents the current player
#[derive(Clone, Copy)]
pub enum Player {
    White,
    Black,
}

struct BoardState {
    current: [[Option<Piece>; 8]; 8],
    opponent: [[Option<Piece>; 8]; 8],
}

impl BoardState {

    pub fn new() -> BoardState {

        let mut boards = Self {
            current: [[None; 8]; 8],
            opponent: [[None; 8]; 8],
        };

        use PieceKind::*;

        boards.current[0][7] = Some(Piece::new(Rook));
        boards.current[1][7] = Some(Piece::new(Knight));
        boards.current[2][7] = Some(Piece::new(Bishop));
        boards.current[3][7] = Some(Piece::new(Queen));
        boards.current[4][7] = Some(Piece::new(King));
        boards.current[5][7] = Some(Piece::new(Bishop));
        boards.current[6][7] = Some(Piece::new(Knight));
        boards.current[7][7] = Some(Piece::new(Rook));

        boards.opponent[0][0] = Some(Piece::new(Rook));
        boards.opponent[1][0] = Some(Piece::new(Knight));
        boards.opponent[2][0] = Some(Piece::new(Bishop));
        boards.opponent[3][0] = Some(Piece::new(Queen));
        boards.opponent[4][0] = Some(Piece::new(King));
        boards.opponent[5][0] = Some(Piece::new(Bishop));
        boards.opponent[6][0] = Some(Piece::new(Knight));
        boards.opponent[7][0] = Some(Piece::new(Rook));

        for x in 0..8 {
            boards.current[x][6] = Some(Piece::new(Pawn));
            boards.opponent[x][1] = Some(Piece::new(Pawn));
        }

        boards
    }

    pub fn get_current(&self, position: Position) -> Result<Option<Piece>, Error> {
        
        if !position.is_valid() {
            Err(Error::InvalidPosition)
        } else {
            let x = position.x as usize;
            let y = position.y as usize;
            Ok(self.current[x][y])
        }
    }

    pub fn get_opponent(&self, position: Position) -> Result<Option<Piece>, Error> {
        
        if !position.is_valid() {
            Err(Error::InvalidPosition)
        } else {
            let x = position.x as usize;
            let y = position.y as usize;
            Ok(self.opponent[x][y])
        }
    }
}

/// Represents a game of chess.
pub struct Game {
    state: State,
    board: BoardState,
    player: Player,
    white_positions: Vec<(PieceKind, Position)>,
    black_positions: Vec<(PieceKind, Position)>,
    moves: Vec<Move>,
    white_points: u32,
    black_points: u32,
    promotion_position: Position,
    last_double_position: Option<Position>,
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
#[derive(Debug)]
pub enum Error {
    /// The method was called while game was in the wrong state.
    InvalidState,
    /// The position provided was outside the bounds of the board.
    InvalidPosition,
}

impl Game {
    
    /// Creates a new game with pieces in inital positions.
    pub fn new() -> Self {
        let mut game = Self {
            state: State::SelectPiece,
            board: BoardState::new(),
            player: Player::White,
            white_positions: Vec::new(),
            black_positions: Vec::new(),
            moves: Vec::new(),
            white_points: 0,
            black_points: 0,
            promotion_position: Position::new(0, 0),
            last_double_position: None,
        };

        game.recalculate_positions();
        game
    }

    /// Resets the game to its original state.
    pub fn reset(&mut self) {
        *self = Self::new();
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

        if !matches!(self.state, State::SelectPiece) {
            return Err(Error::InvalidState);
        }

        if !position.is_valid() {
            return Err(Error::InvalidPosition);
        }

        match self.board.get_current(position).unwrap() {
            None => Ok(()),
            Some(_) => {
                self.state = State::SelectMove;
                self.calculate_moves(position);
                Ok(())
            },
        }
    }

    /// Get a slice of legal moves for piece selected with [Game::select_piece].
    /// Returns [Error::InvalidState] if game state is not [State::SelectMove].
    pub fn get_moves(&self) -> Result<&[Move], Error> {

        if !matches!(self.state, State::SelectMove) {
            return Err(Error::InvalidState);
        }

        Ok(&self.moves[..])
    }

    /// Selects move corresponding to `position`.
    /// Returns [Error::InvalidState] if game state is not [State::SelectMove].
    /// Returns [Error::InvalidPosition] position is not on the board.
    /// If `position` doesn't correspond to a legal move, 
    /// the state is reverted back to [State::SelectPiece].
    pub fn select_move(&mut self, position: Position) -> Result<(), Error> {

        if !position.is_valid() {
            return Err(Error::InvalidPosition);
        }

        if !matches!(self.state, State::SelectMove) {
            return Err(Error::InvalidState);
        }

        // Find move with provided position
        let mov = self.moves.iter()
            .find(|m| m.to == position);

        match mov {

            None => self.state = State::SelectPiece,
            Some(mov) => self.execute_move(*mov),
        };

        Ok(())
    }

    /// Selects a piece to promote to.
    /// Returns [Error::InvalidState] if game state is not [State::SelectPromotion].
    /// If `promotion` is [None], the piece is not promoted.
    pub fn select_promotion(&mut self, promotion: Option<Promotion>) -> Result<(), Error> {

        if !matches!(self.state, State::SelectPromotion) {
            return Err(Error::InvalidState);
        }

        self.state = State::SelectPiece;

        let mut piece = match promotion {

            // No promotion
            None => {
                self.swap_players();
                return Ok(());
            },

            // Create promoted piece
            Some(kind) => Piece::from_promotion(kind),
        };

        // Can't do castling with this one
        piece.moved = true;

        // Promote
        let x = self.promotion_position.x as usize;
        let y = self.promotion_position.y as usize;
        self.board.current[x][y] = Some(piece);

        // New turn
        self.swap_players();
        self.recalculate_positions();

        Ok(())
    }

    /// Returns a slice of black pieces with corresponding positions.
    pub fn get_black_positions(&self) -> &[(PieceKind, Position)] {
        &self.black_positions[..]
    }

    /// Returns a slice of white pieces with corresponding positions.
    pub fn get_white_positions(&self) -> &[(PieceKind, Position)] {
        &self.white_positions[..]
    }


    fn recalculate_positions(&mut self) {

        let (white_board, black_board,) = match self.player {
            Player::White => (&self.board.current, &self.board.opponent,),
            Player::Black => (&self.board.opponent, &self.board.current,),
        };

        self.white_positions.clear();
        self.black_positions.clear();

        for (x, y) in std::iter::zip(0usize..8, 0usize..8) {

            let ix = x as isize;
            let iy = y as isize;

            if let Some(piece) = white_board[x][y] {
                self.white_positions.push((piece.kind, Position::new(ix, iy)));
            }

            if let Some(piece) = black_board[x][y] {
                self.black_positions.push((piece.kind, Position::new(ix, iy)));
            }
        }
    }

    fn add_point(&mut self) {

        match self.player {
            Player::White => self.white_points += 1,
            Player::Black => self.black_points += 1,
        };
    }

    fn swap_players(&mut self) {

        (self.board.current, self.board.opponent,) =
            (self.board.opponent, self.board.current,);
    }

    fn calculate_moves(&mut self, position: Position) {

        self.moves.clear();

        if let Some(piece) = self.board.get_current(position).unwrap() {

            match piece.kind {

                PieceKind::Pawn => self.calculate_pawn(position),
                PieceKind::Rook => self.calculate_rook(piece.moved, position),
                PieceKind::Knight => self.calculate_knight(position),
                PieceKind::Bishop => self.calculate_bishop(position),
                PieceKind::Queen => self.calculate_queen(position),
                PieceKind::King => self.calculate_king(piece.moved, position),
            };
        }
    }

    fn execute_move(&mut self, mov: Move) {

        self.last_double_position = None;

        match mov.kind {
            MoveKind::Regular => {}, 
            MoveKind::Double => {},
            MoveKind::Promotion => {},
            MoveKind::EnPassant => {},
            MoveKind::Castling { other_pos } => {},
       };

       self.recalculate_positions(); 

       if matches!(mov.kind, MoveKind::Promotion) {
           self.state = State::Promotion;
       } else {
            
           self.swap_players();

           if (self.is_checkmated()) {
               self.state = State::CheckMate;
           } else {
               self.state = State::SelectPiece;
           }
       }
    }

    fn calculate_pawn(&self, position: Position) { unimplemented!() }

    fn calculate_rook(&self, moved: bool, position: Position) { unimplemented!() }

    fn calculate_knight(&self, position: Position) { unimplemented!() }

    fn calculate_bishop(&self, position: Position) { unimplemented!() }

    fn calculate_queen(&self, position: Position) { unimplemented!() }

    fn calculate_king(&self, moved: bool, position: Position) { unimplemented!() }

    fn try_move(&self, mov: Move) -> Option<Piece> {
        
        
    }
}
