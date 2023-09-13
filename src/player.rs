
#[derive(Clone, Copy)]
pub enum Player { White, Black, }

impl Default for Player {
    fn default() -> Self { Player::White }
}
