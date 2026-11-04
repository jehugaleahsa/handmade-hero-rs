#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Directions {
    flags: u8,
}

impl Directions {
    const UP_FLAG: u8 = 1;
    const RIGHT_FLAG: u8 = 2;
    const DOWN_FLAG: u8 = 4;
    const LEFT_FLAG: u8 = 8;

    #[inline]
    #[must_use]
    pub fn up() -> Self {
        let flags = Self::UP_FLAG;
        Self { flags }
    }

    #[inline]
    #[must_use]
    pub fn right() -> Self {
        let flags = Self::RIGHT_FLAG;
        Self { flags }
    }

    #[inline]
    #[must_use]
    pub fn down() -> Self {
        let flags = Self::DOWN_FLAG;
        Self { flags }
    }

    #[inline]
    #[must_use]
    pub fn left() -> Self {
        let flags = Self::LEFT_FLAG;
        Self { flags }
    }

    #[inline]
    #[must_use]
    pub fn is_up(self) -> bool {
        self.is_direction(Self::UP_FLAG)
    }

    #[inline]
    #[must_use]
    pub fn is_right(self) -> bool {
        self.is_direction(Self::RIGHT_FLAG)
    }

    #[inline]
    #[must_use]
    pub fn is_down(self) -> bool {
        self.is_direction(Self::DOWN_FLAG)
    }

    #[inline]
    #[must_use]
    pub fn is_left(self) -> bool {
        self.is_direction(Self::LEFT_FLAG)
    }

    #[inline]
    #[must_use]
    fn is_direction(self, flag: u8) -> bool {
        self.flags & flag != 0
    }

    #[inline]
    #[must_use]
    pub fn any(self, other: Self) -> bool {
        self.flags & other.flags != 0
    }

    #[inline]
    #[must_use]
    pub fn all(self, other: Self) -> bool {
        self.flags & other.flags == other.flags
    }

    #[inline]
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        let flags = self.flags | other.flags;
        Self { flags }
    }

    #[inline]
    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        let flags = self.flags & other.flags;
        Self { flags }
    }

    #[inline]
    #[must_use]
    pub fn difference(self, other: Self) -> Self {
        let flags = self.flags & !other.flags;
        Self { flags }
    }
}
