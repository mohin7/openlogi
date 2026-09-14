/// A minimal bitflags generator.
///
/// HID++ is full of small flag bytes. Pulling in the `bitflags` crate for
/// three types would be the only dependency in this crate that isn't earning
/// its place, so this macro provides just the surface we use: `empty`,
/// `from_bits_truncate`, `contains`, `bits`, and `|`/`|=`.
#[macro_export]
macro_rules! bitflags_lite {
    (
        $(#[$meta:meta])*
        pub struct $name:ident : $ty:ty {
            $( $(#[$fmeta:meta])* const $flag:ident = $value:expr; )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct $name($ty);

        impl $name {
            $( $(#[$fmeta])* pub const $flag: Self = Self($value); )*

            pub const fn empty() -> Self { Self(0) }

            pub const fn all() -> Self { Self(0 $(| $value)*) }

            /// Build from raw bits, silently dropping bits we do not define.
            /// Truncating rather than failing matters here: newer firmware
            /// sets flags this build has never heard of, and dropping an
            /// unknown bit is always better than refusing to read the device.
            pub const fn from_bits_truncate(bits: $ty) -> Self {
                Self(bits & Self::all().0)
            }

            pub const fn bits(self) -> $ty { self.0 }

            pub const fn contains(self, other: Self) -> bool {
                (self.0 & other.0) == other.0
            }

            pub const fn intersects(self, other: Self) -> bool {
                (self.0 & other.0) != 0
            }

            pub const fn is_empty(self) -> bool { self.0 == 0 }
        }

        impl core::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self { Self(self.0 | rhs.0) }
        }

        impl core::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) { self.0 |= rhs.0; }
        }

        impl core::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self { Self(self.0 & rhs.0) }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let mut first = true;
                $(
                    if self.contains(Self::$flag) {
                        if !first { f.write_str("|")?; }
                        f.write_str(stringify!($flag))?;
                        first = false;
                    }
                )*
                if first { f.write_str("(none)")?; }
                Ok(())
            }
        }
    };
}
