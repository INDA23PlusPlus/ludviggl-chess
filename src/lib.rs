
//! # Chess backend
//! ## Usage
//! All game logic is handled by [Game] struct.
//! Information can be queried with the functions:
//! * [Game::get_state]: get the current [State] of the game.
//! * [Game::get_current_player]: get the current [Player].
//! * [Game::get_black_positions]/[Game::get_white_positions]: get pieces and corresponding
//! positions.
//! * [Game::get_moves]: get all destination positions corresponding to legal moves for piece
//! previously selected with [Game::select_piece].
//!
//! Some methods are associated with a certain state, and returns [Error::InvalidState] if called when game is
//! in a different state. These methods are:
//! * [Game::select_piece]: may only be called when game state is [State::SelectPiece].
//! * [Game::get_moves]: may only be called when game state is [State::SelectMove].
//! * [Game::select_move]: may only be called when game state is [State::SelectMove].
//!
//! ## Examples
//! Functions not in this crate or prepended with `frontend::`,
//! and are just example functions.
//! ### Initializing
//! ```ignore
//! let mut game = Game::new();
//! ```
//! ### Rendering
//! ```ignore
//! let mut game = Game::new();
//! for (piece, x, y) in game.get_white_positions() {
//!     frontend::render_white_piece(piece, x, y);
//! }
//!
//! for (piece, x, y) in game.get_black_positions() {
//!     frontend::render_black_piece(piece, x, y);
//! }
//!
//! match game.get_state() {
//!     State::SelectMove => {
//!         for (x, y) in game.get_moves() {
//!             frontend::highlight_square(x, y);
//!         }
//!     },
//!     _ => (),
//! };
//! ```
//! ### Game logic
//! ```ignore
//! // assuming frontend::get_clicked_square() only returns valid positions:
//! match game.get_state() {
//!     State::SelectPiece => {
//!         let (x, y) = frontend::get_clicked_square();
//!         game.select_piece(x, y).unwrap(); // we know state is State::SelectPiece
//!                                           // and position is valid, hence .unwrap()
//!     },
//!     State::SelectMove => {
//!         let (x, y) = frontend::get_clicked_square();
//!         game.select_move(x, y).unwrap(); // we know state is State::SelectMove
//!                                           // and position is valid, hence .unwrap()
//!     }
//! }
//! ```

#[macro_use]
extern crate lazy_static;

mod piece;
mod player;
mod game;
mod board;
mod utils;
mod moves;
mod error;

pub use piece::Piece;
pub use player::Player;
pub use game::{ Game, State, };
pub use error::Error;
