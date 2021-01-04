#![no_std]

use core::{
    cmp::min,
    mem::size_of,
    ops::{Add, BitAnd, BitOr},
};

#[derive(PartialEq, Debug, Eq, Copy, Clone)]
pub struct NanoBV<T = u32> {
    data: T,
    length: usize,
}

impl<T> NanoBV<T> {
    pub const fn new(data: T, length: usize) -> Self {
        NanoBV { data, length }
    }

    pub const fn len(&self) -> usize {
        self.length
    }
}

macro_rules! ImplNanoBVCommon {
    (for $($type:tt),+) => {
        $(ImplNanoBVCommon!($type);)*
    };

    ($type:ident) => {
        impl NanoBV<$type> {
            const BIT_SIZE: usize = size_of::<$type>() * 8;

            const fn max(length: usize) -> $type {
                match length {
                    Self::BIT_SIZE => $type::MAX,
                    _ => (1 << length) - 1,
                }
            }

            pub const fn default() -> Self {
                NanoBV::new($type::MIN, Self::BIT_SIZE)
            }

            pub const fn value(&self) -> $type {
                self.data
            }

            pub const fn set_value(&self, value: $type) -> Self {
                let new_value = value & Self::max(self.length);
                NanoBV::new(new_value, self.length)
            }

            pub const fn zeros(length: usize) -> Self {
                NanoBV::new(0, length)
            }

            pub const fn ones(length: usize) -> Self {
                NanoBV::new(Self::max(length), length)
            }

            pub const fn clear(&self) -> Self {
                NanoBV::new(0, self.length)
            }

            pub const fn set(&self) -> Self {
                NanoBV::new(Self::max(self.length), self.length)
            }

            pub const fn get_bit(&self, offset: $type) -> $type {
                (self.data >> offset) & 1
            }

            pub const fn set_bit(&self, offset: $type) -> Self {
                let new_value = self.data | (1 << offset) & Self::max(self.length);
                NanoBV::new(new_value, self.length)
            }

            pub const fn clear_bit(&self, offset: $type) -> Self {
                let new_value = self.data & !(1 << offset);
                NanoBV::new(new_value, self.length)
            }

            pub const fn assign_bit(&self, value: $type, offset: $type) -> Self {
                match value {
                0 => self.clear_bit(offset),
                _ => self.set_bit(offset),
                }
            }

            pub const fn reverse(&self) -> NanoBV<$type> {
                let mut reversed = self.data.reverse_bits();
                reversed >>= Self::BIT_SIZE - self.length;
                NanoBV::new(reversed, self.length)
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
        impl<T: $trait + $trait<Output = T>> $trait for NanoBV<T> {
            type Output = Self;

            fn $function(self, other: Self) -> Self {
                Self::new(self.data.$function(other.data), min(self.length, other.length))
            }
        }
    };
}

ImplNanoBVOps!(for (Add, add), (BitAnd, bitand), (BitOr, bitor));

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
            fn [<test_nanobv_reverse_ $type>]() {
                type NBV = NanoBV::<$type>;
                let mut rng = RNG::<WyRand, $type>::new(0xDEADBEEF);
                let data = rng.generate();
                let bv = NBV::new(data, NBV::BIT_SIZE);
                assert_eq!(bv.reverse().value(), data.reverse_bits());
            }
        }
        };
    }

    ImplNanoBVTest!(for u8, u16, u32, u64);
}
