
/// Represents the current player.
#[derive(Clone, Copy, Debug)]
pub enum Player { White, Black, }

impl Default for Player {
    fn default() -> Self { Player::White }
}
