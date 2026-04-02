use bevy::prelude::*;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

#[derive(Component, Resource, Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[reflect(Component, Resource, Default)]
pub struct GameFeelChannels(pub u64);

impl GameFeelChannels {
    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(u64::MAX);
    pub const GAMEPLAY: Self = Self(1 << 0);
    pub const AMBIENT: Self = Self(1 << 1);
    pub const WEAPON: Self = Self(1 << 2);
    pub const UI: Self = Self(1 << 3);

    #[must_use]
    pub const fn new(bits: u64) -> Self {
        Self(bits)
    }

    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    #[must_use]
    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl Default for GameFeelChannels {
    fn default() -> Self {
        Self::ALL
    }
}

impl BitOr for GameFeelChannels {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for GameFeelChannels {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for GameFeelChannels {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for GameFeelChannels {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for GameFeelChannels {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
