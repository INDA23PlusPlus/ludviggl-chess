

/// Error variants for [crate::Game] methods.
pub enum Error {
    /// The method was called in the incorrect state.
    InvalidState,
    /// The position provided lies outside the board.
    InvalidPosition,
}
