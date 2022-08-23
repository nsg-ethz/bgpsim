// NetSim: BGP Network Simulator written in Rust
// Copyright (C) 2022 Tibor Schneider
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

//! This module contains the definition for prefixes. In addition, it contains all collections
//! containing the prefix. This allows consistent handling of the feature `multi_prefix`.

pub use _prefix::*;

#[cfg(feature = "multi_prefix")]
mod _prefix {
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    /// IP Prefix (simple representation)
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Prefix(pub u32);

    impl std::fmt::Display for Prefix {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Prefix({})", self.0)
        }
    }

    impl From<u32> for Prefix {
        fn from(x: u32) -> Self {
            Self(x)
        }
    }

    impl From<u64> for Prefix {
        fn from(x: u64) -> Self {
            Self(x as u32)
        }
    }

    impl From<usize> for Prefix {
        fn from(x: usize) -> Self {
            Self(x as u32)
        }
    }

    impl From<i32> for Prefix {
        fn from(x: i32) -> Self {
            Self(x as u32)
        }
    }

    impl From<i64> for Prefix {
        fn from(x: i64) -> Self {
            Self(x as u32)
        }
    }

    impl From<isize> for Prefix {
        fn from(x: isize) -> Self {
            Self(x as u32)
        }
    }

    impl<T> From<&T> for Prefix
    where
        T: Into<Prefix> + Copy,
    {
        fn from(x: &T) -> Self {
            (*x).into()
        }
    }
}

#[cfg(not(feature = "multi_prefix"))]
mod _prefix {
    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    /// IP Prefix with zero-size.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Prefix;

    impl std::fmt::Display for Prefix {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Prefix")
        }
    }

    impl From<u32> for Prefix {
        fn from(_: u32) -> Self {
            Self
        }
    }

    impl From<u64> for Prefix {
        fn from(_: u64) -> Self {
            Self
        }
    }

    impl From<usize> for Prefix {
        fn from(_: usize) -> Self {
            Self
        }
    }

    impl From<i32> for Prefix {
        fn from(_: i32) -> Self {
            Self
        }
    }

    impl From<i64> for Prefix {
        fn from(_: i64) -> Self {
            Self
        }
    }

    impl From<isize> for Prefix {
        fn from(_: isize) -> Self {
            Self
        }
    }

    impl<T> From<&T> for Prefix
    where
        T: Into<Prefix> + Copy,
    {
        fn from(_: &T) -> Self {
            Self
        }
    }
}
