#![no_std]

use core::{
    cmp::min,
    convert::{TryFrom, TryInto},
    mem::size_of,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub},
};

#[derive(PartialEq, Debug, Eq, Copy, Clone)]
pub struct NanoBV<T = u32> {
    data: T,
    length: usize,
}

impl<T> NanoBV<T> {
    /// Retrieve length of the current NanoBV.
    pub const fn len(&self) -> usize {
        self.length
    }

    #[doc(hidden)]
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }
}

macro_rules! ImplNanoBVCommon {
    (for $($type:tt),+) => {
        $(ImplNanoBVCommon!($type);)*
    };

    ($type:ident) => {
        impl NanoBV<$type> {
            const BIT_SIZE: usize = size_of::<$type>() * 8;

            const fn upper_bound(length: usize) -> $type {
                match length {
                    Self::BIT_SIZE => $type::MAX,
                    _ => (1 << length) - 1,
                }
            }

            /// Create a new [`NanoBV`].
            pub const fn new(data: $type, length: usize) -> Self {
                NanoBV { data: data & Self::upper_bound(length), length }
            }

            /// Create a [`NanoBV`] initialized to 0 with length equivalent to the size of the
            /// stored type.
            pub const fn default() -> Self {
                NanoBV::<$type>::new($type::MIN, Self::BIT_SIZE)
            }

            /// Retrieve value of the current NanoBV.
            pub const fn value(&self) -> $type {
                self.data
            }

            /// Set value of the current NanoBV while retaining length.
            pub const fn set_value(&self, value: $type) -> Self {
                let new_value = value & Self::upper_bound(self.length);
                NanoBV::<$type>::new(new_value, self.length)
            }

            /// Create [`NanoBV`] with all bits unset.
            pub const fn zeros(length: usize) -> Self {
                NanoBV::<$type>::new(0, length)
            }

            /// Create [`NanoBV`] with all bits set.
            pub const fn ones(length: usize) -> Self {
                NanoBV::<$type>::new(Self::upper_bound(length), length)
            }

            /// Clear all bits.
            pub const fn clear(&self) -> Self {
                Self::zeros(self.length)
            }

            /// Set all bits.
            pub const fn set(&self) -> Self {
                Self::ones(self.length)
            }

            /// Get bit at offset.
            pub const fn get_bit(&self, offset: $type) -> $type {
                (self.data >> offset) & 1
            }

            /// Set bit at offset.
            pub const fn set_bit(&self, offset: $type) -> Self {
                let new_value = self.data | (1 << offset) & Self::upper_bound(self.length);
                NanoBV::<$type>::new(new_value, self.length)
            }

            /// Clear bit at offset.
            pub const fn clear_bit(&self, offset: $type) -> Self {
                let new_value = self.data & !(1 << offset);
                NanoBV::<$type>::new(new_value, self.length)
            }

            /// Assign bit at offset.
            pub const fn assign_bit(&self, value: $type, offset: $type) -> Self {
                match value {
                0 => self.clear_bit(offset),
                _ => self.set_bit(offset),
                }
            }

            /// Reverse bits.
            pub const fn reverse(&self) -> NanoBV<$type> {
                let mut reversed = self.data.reverse_bits();
                reversed >>= (Self::BIT_SIZE - self.length) as $type;
                NanoBV::<$type>::new(reversed, self.length)
            }
        }
    };
}

ImplNanoBVCommon!(for u8, u16, u32, u64);

macro_rules! ImplNanoBVOps {
    (for $(($trait:tt, $function:tt)),+) => {
        $(ImplNanoBVOps!($trait, $function);)*
    };

    ($trait:ident, $function:ident) => {
        impl<T: $trait + $trait<Output = T> + TryFrom<u128> + TryInto<u128> + Default> $trait for NanoBV<T> {
            type Output = Self;

            fn $function(self, other: Self) -> Self {
                let length = min(self.length, other.length);
                let data = self.data.$function(other.data).try_into().unwrap_or_default() & ((1u128 << length) - 1);
                Self { data: T::try_from(data).unwrap_or_default(), length }
            }
        }
    };
}

ImplNanoBVOps!(for (Add, add), (BitAnd, bitand), (BitOr, bitor), (BitXor, bitxor), (Div, div), (Mul, mul), (Rem, rem), (Shl, shl), (Shr, shr), (Sub, sub));

#[cfg(test)]
mod tests {
    use super::*;
    use paste::paste;
    use picorand::{PicoRandGenerate, WyRand, RNG};

    macro_rules! ImplNanoBVTest {
        (for $($type:tt),+) => {
            $(ImplNanoBVTest!($type);)*
        };

        ($type:ident) => {
        paste! {
            #[test]
            fn [<test_nanobv_zeros_ $type>]() {
                type NBV = NanoBV::<$type>;
                let bv = NBV::zeros(NBV::BIT_SIZE);
                assert_eq!(bv.value(), $type::MIN);
                assert_eq!(bv.len(), NBV::BIT_SIZE);
            }

            #[test]
            fn [<test_nanobv_ones_ $type>]() {
                type NBV = NanoBV::<$type>;
                let bv = NBV::ones(NBV::BIT_SIZE);
                assert_eq!(bv.value(), $type::MAX);
                assert_eq!(bv.len(), NBV::BIT_SIZE);
            }

            #[test]
            fn [<test_nanobv_default_ $type>]() {
                type NBV = NanoBV::<$type>;
                let bv = NBV::default();
                assert_eq!(bv, NBV::zeros(NBV::BIT_SIZE));
            }

            #[test]
            fn [<test_nanobv_get_bit_ $type>]() {
                type NBV = NanoBV::<$type>;
                let bv = NBV::new($type::MAX, NBV::BIT_SIZE);
                let mut rng = RNG::<WyRand, $type>::new($type::MAX as _);
                let offset = rng.generate_range(0, NBV::BIT_SIZE);
                assert_eq!(bv.get_bit(offset), 1);
            }

            #[test]
            fn [<test_nanobv_set_bit_ $type>]() {
                type NBV = NanoBV::<$type>;

                let mut rng = RNG::<WyRand, $type>::new($type::MAX as _);
                let offset = rng.generate_range(0, NBV::BIT_SIZE);
                let bv = NBV::new($type::MIN, NBV::BIT_SIZE).set_bit(offset);
                assert_eq!(bv.get_bit(offset), 1);
            }

            #[test]
            fn [<test_nanobv_clear_bit_ $type>]() {
                type NBV = NanoBV::<$type>;

                let mut rng = RNG::<WyRand, $type>::new($type::MAX as _);
                let offset = rng.generate_range(0, NBV::BIT_SIZE);
                let bv = NBV::new($type::MAX, NBV::BIT_SIZE).clear_bit(offset);
                assert_eq!(bv.get_bit(offset), 0);
            }

            #[test]
            fn [<test_nanobv_assign_bit_ $type>]() {
                type NBV = NanoBV::<$type>;

                let mut rng = RNG::<WyRand, $type>::new($type::MAX as _);
                let offset = rng.generate_range(0, NBV::BIT_SIZE);
                let value = rng.generate_range(0, 2);
                let bv = NBV::new($type::MAX, NBV::BIT_SIZE).assign_bit(value, offset);
                assert_eq!(bv.get_bit(offset), value);
            }

            #[test]
            fn [<test_nanobv_reverse_ $type>]() {
                type NBV = NanoBV::<$type>;
                let mut rng = RNG::<WyRand, $type>::new($type::MAX as _);
                let data = rng.generate();
                let bv = NBV::new(data, NBV::BIT_SIZE);
                assert_eq!(bv.reverse().value(), data.reverse_bits());
            }
        }
        };
    }

    ImplNanoBVTest!(for u8, u16, u32, u64);
}
