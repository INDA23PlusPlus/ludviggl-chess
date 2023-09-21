

/// Error variants for [crate::Game] methods.
#[derive(Debug)]
pub enum Error {
    /// The method was called in the incorrect state.
    InvalidState,
    /// The position provided lies outside the board.
    InvalidPosition,
    /// The piece provided is not a valid promotion.
    InvalidPromotion,
}
