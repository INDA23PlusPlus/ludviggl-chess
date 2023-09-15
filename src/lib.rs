
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
//! * [Game::get_selected_pos]: get position of piece selected with [Game::select_piece].
//!
//! Some methods are associated with a certain state, and returns [Error::InvalidState] if called when game is
//! in a different state. These methods are:
//! * [Game::select_piece]: may only be called when game state is [State::SelectPiece].
//! * [Game::get_moves]: may only be called when game state is [State::SelectMove].
//! * [Game::get_selected_piece]: may only be called when game state is [State::SelectMove].
//! * [Game::select_move]: may only be called when game state is [State::SelectMove].
//!
//! ## Examples
//! Functions not in this crate or prepended with `frontend::`,
//! and are just example functions.
//! ### Initializing
//! ```no_run
//! use ludviggl_chess::Game;
//! let mut game = Game::new();
//! ```
//! ### Rendering
//! ```no_run
//! use ludviggl_chess::{ Game, Piece, State, };
//!
//! # mod frontend {
//! # use ludviggl_chess::Piece;
//! # pub fn render_white_piece(_p: Piece, _x: u8, _y: u8) {}
//! # pub fn render_black_piece(_p: Piece, _x: u8, _y: u8) {}
//! # pub fn highlight_square(_x: u8, _y: u8) {}
//! # }
//! # let mut game = Game::new();
//! for &(piece, x, y) in game.get_white_positions() {
//!     frontend::render_white_piece(piece, x, y);
//! }
//!
//! for &(piece, x, y) in game.get_black_positions() {
//!     frontend::render_black_piece(piece, x, y);
//! }
//!
//! match game.get_state() {
//!     State::SelectMove => {
//!         if let Ok(moves) = game.get_moves() {
//!             for &(x, y) in moves {
//!                 frontend::highlight_square(x, y);
//!             }
//!         }
//!     },
//!     _ => (),
//! };
//! ```
//! ### Game logic
//! ```no_run
//! use ludviggl_chess::{ Game, Piece, State, };
//!
//! # mod frontend {
//! #    pub fn get_clicked_square() -> (u8, u8) { (0, 0,) }
//! #    pub fn game_over() {}
//! # }
//! # let mut game = Game::new();
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
//!     },
//!     State::CheckMate => {
//!         frontend::game_over();
//!     },
//! }
//! ```

#[macro_use]
extern crate lazy_static;

pub mod piece;
pub mod player;
pub mod game;
mod board;
#[allow(dead_code)]
mod utils;
mod moves;
pub mod error;

pub use piece::Piece;
pub use player::Player;
pub use game::{ Game, State, };
pub use error::Error;
