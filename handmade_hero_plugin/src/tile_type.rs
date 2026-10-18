use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum TileType {
    #[default]
    Floor,
    Door,
    Wall,
    UpStairs,
    DownStairs,
}

impl TileType {
    pub fn is_traversable(self) -> bool {
        match self {
            Self::Floor | Self::Door | Self::UpStairs | Self::DownStairs => true,
            Self::Wall => false,
        }
    }

    pub fn is_stairs(self) -> bool {
        matches!(self, Self::UpStairs | Self::DownStairs)
    }
}
