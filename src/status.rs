use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

pub struct Status(pub i32);

#[inline]
pub const fn i32_bit_mask(low: i32, high: i32) -> i32 {
    assert!(low >= 0);
    assert!(low <= high);
    assert!(high < 32);
    if high == 31 {
        0
    } else {
        (1 << high + 1) - (1 << low)
    }
}

impl Status {
    const RESPONSE_CODE_BITMASK: i32 = i32_bit_mask(0, 9);
    pub const REQUEST_CONSUMED: Self = Self(1 << 10);
    pub const RESPONSE_CONSUMED: Self = Self(1 << 11);
    pub const REQUEST_LISTENERS_COMPLETE: Self = Self(1 << 12);
    pub const RESPONSE_LISTENERS_COMPLETE: Self = Self(1 << 13);
    pub const REQUEST_BUFFERED: Self = Self(1 << 14);
    pub const RESPONSE_BUFFERED: Self = Self(1 << 15);

    pub fn any_flags(&self, flags: Status) -> bool {
        self.0 & flags.0 != 0
    }

    pub fn any_flags_clear(&self, flags: Status) -> bool {
        self.0 & flags.0 != flags.0
    }

    pub fn all_flags(&self, flags: Status) -> bool {
        self.0 & flags.0 == 0
    }

    pub fn all_flags_clear(&self, flags: Status) -> bool {
        self.0 & flags.0 == 0
    }
}

impl PartialEq for Status {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl BitOrAssign for Status {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitAndAssign for Status {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl Not for Status {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitAnd for Status {
    type Output = Self;
    fn bitand(
        self,
        rhs: Self
    ) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for Status {
    type Output = Self;
    fn bitor(
        self,
        rhs: Self
    ) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}