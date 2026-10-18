use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum TileType {
    #[default]
    Floor,
    Door,
    Wall,
    Stairs,
}

impl TileType {
    pub fn is_traversable(self) -> bool {
        match self {
            Self::Floor | Self::Door | Self::Stairs => true,
            Self::Wall => false,
        }
    }
}
