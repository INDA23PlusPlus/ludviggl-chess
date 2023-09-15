
use crate::{
    error::Error,
    piece::Piece,
    player::Player,
    board::Board,
    utils,
};

/// Struct containing all game state and data.
pub struct Game {
    state: State,
    board: Board,
    selected_pos: (u8, u8),
    selected_id: usize,
    selected_moves: (u64, Vec<(u8, u8)>),
    black_positions: Vec<(Piece, u8, u8)>,
    white_positions: Vec<(Piece, u8, u8)>,
}

/// Represents the current state of the game.
#[derive(Clone, Copy)]
pub enum State {
    /// Current player needs to select a piece to move.
    SelectPiece,
    /// Current player needs to select a move to play for select piece.
    SelectMove,
    /// Current player is in checkmate.
    CheckMate,
}

impl Game {

    /// Creates a new game with pieces in inital positions.
    pub fn new() -> Game {
        let mut game = Game {
            state: State::SelectPiece,
            board: Board::new(),
            selected_pos: (0, 0),
            selected_id: 0,
            selected_moves: (0, Vec::new()),
            black_positions: Vec::new(),
            white_positions: Vec::new(),
        };

        game.update_positions();
        game
    }

    /// Resets the game to its initial state
    pub fn reset(&mut self) {
        *self = Game::new();
    }

    /// Returns the state of the game.
    pub fn get_state(&self) -> State {
        self.state
    }
    
    /// Returns the player whos turn it is.
    pub fn get_current_player(&self) -> Player {
        self.board.player
    }

    /// Returns black pieces and their positions
    pub fn get_black_positions(&self) -> &[(Piece, u8, u8)] {
        &self.black_positions[..]
    }

    /// Returns white pieces and their positions
    pub fn get_white_positions(&self) -> &[(Piece, u8, u8)] {
        &self.white_positions[..]
    }

    /// Selects a piece by position on the board.
    /// If position is occupied by the current player, transitions state to [State::SelectMove].
    /// If position is empty or occupied by opponent, does nothing.
    /// Returns [Error::InvalidState] if game state is not [State::SelectPiece].
    /// Returns [Error::InvalidPosition] if position is not on the board.
    pub fn select_piece(&mut self, x: u8, y: u8) -> Result<(), Error> { 

        if !matches!(self.state, State::SelectPiece) {
            return Err(Error::InvalidState);
        }

        if !valid_pos(x, y) {
            return Err(Error::InvalidPosition);
        }

        self.selected_moves.0 = 0;
        self.selected_moves.1.clear();

        match self.board.id_from_pos(x, y) {
            None => (), // no piece at pos
            Some(id) => {
                    self.selected_pos = (x, y);
                    self.selected_id = id;
                    self.state = State::SelectMove;

                    match self.board.get_legal_moves(id) {
                        0 => (), // no legal moves
                        m => {
                            self.selected_moves.0 = m;
                            self.selected_moves.1 = utils::BitIterator::new(m)
                                                    .map(|x| utils::unflatten_bit(x))
                                                    .collect::<Vec<_>>();
                        }
                    };
            },
        };
        Ok(())
    }

    /// Returns positions corresponding to the legal moves for piece selected with
    /// [Game::select_piece]. Can be empty slice.
    /// Returns [Error::InvalidState] if game state is not [State::SelectMove].
    pub fn get_moves(&self) -> Result<&[(u8, u8)], Error> {

        if !matches!(self.state, State::SelectMove) {
            return Err(Error::InvalidState);
        }

        Ok(&self.selected_moves.1[..])
    }

    /// Returns position of currently selected piece.
    /// Returns [Error::InvalidState] if game state is not [State::SelectMove].
    pub fn get_selected_pos(&self) -> Result<(u8, u8), Error> {

        if !matches!(self.state, State::SelectMove) {
            return Err(Error::InvalidState);
        }

        Ok(self.selected_pos)
    }

    /// Selects a move by corresponding position and executes it.
    /// If position does not correspond to a legal move, reverts state
    /// back to [State::SelectPiece].
    /// Returns [Error::InvalidPosition] if position is not on the board.
    /// Returns [Error::InvalidState] if game state is not [State::SelectMove].
    pub fn select_move(&mut self, x: u8, y: u8) -> Result<(), Error> {

        if !matches!(self.state, State::SelectMove) {
            return Err(Error::InvalidState);
        }

        if !valid_pos(x, y) {
            return Err(Error::InvalidPosition);
        }

        let dest = utils::flatten_bit(x, y);

        if dest & self.selected_moves.0 > 0 {
            self.board.play_move(self.selected_id, dest);
        }

        self.state = State::SelectPiece;

        self.update_positions();

        if self.board.is_checkmate() {
            self.state = State::CheckMate;
        }

        Ok(())
    }

    fn update_positions(&mut self) {
        self.black_positions = self.board.black_iter().collect();
        self.white_positions = self.board.white_iter().collect();
    }

}

fn valid_pos(x: u8, y: u8) -> bool {
    x < 8 && y < 8
}
