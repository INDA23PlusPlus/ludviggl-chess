
use crate::{
    error::Error,
    piece::Piece,
    player::Player,
};


pub struct Game {
    state: State,
}

#[derive(Clone, Copy)]
pub enum State {
    SelectPiece,
    SelectMove,
}

impl Game {

    pub fn new() -> Game { unimplemented!() }

    pub fn get_state(&self) -> State {
        self.state
    }

    pub fn get_current_player(&self) -> Player { unimplemented!() }

    pub fn pick_piece(&mut self, x: u8, y: u8) -> Result<(), Error> { unimplemented!() }

    pub fn get_moves(&mut self) -> Result<&[(u8, u8)], Error> { unimplemented!() }

    pub fn pick_move(&mut self, x: u8, y: u8) -> Result<(), Error> { unimplemented!() }

    pub fn pick_promotion(&mut self, piece: Piece) -> Result<(), Error> { unimplemented!() }

}
