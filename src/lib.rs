use core::mem::size_of;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
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

            pub const fn default() -> Self {
                NanoBV::new($type::MIN, size_of::<$type>() * 8)
            }

            pub const fn value(&self) -> $type {
                self.data
            }

            const fn max(length: usize) -> $type {
                if length == NanoBV::<$type>::BIT_SIZE {
                return $type::MAX;
                }
                (1 << length) - 1
            }

            pub const fn set_value(mut self, value: $type) {
                self.data = value & Self::max(self.length);
            }

            pub const fn clear(mut self) {
                self.data = 0;
            }

            pub const fn set(mut self) {
                self.data = Self::max(self.length);
            }

            pub const fn zeros(length: usize) -> Self {
                NanoBV::new(0, length)
            }

            pub const fn ones(length: usize) -> Self {
                NanoBV::<$type>::new(Self::max(length), length)
            }

            pub const fn reverse(&self) -> NanoBV<$type> {
                let mut reversed = self.data.reverse_bits();
                reversed >>= (size_of::<$type>() * 8) - self.length;
                NanoBV::<$type>::new(reversed, self.length)
            }
        }
    };
}

ImplNanoBVCommon!(for u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;
    use paste::paste;

    macro_rules! ImplNanoBVTest {
        (for $($type:tt),+) => {
            $(ImplNanoBVTest!($type);)*
        };

        ($type:ident) => {
        paste! {
            #[test]
            fn [<test_nanobv_zeros_ $type>]() {
                let bv = NanoBV::<$type>::zeros(size_of::<$type>() * 8);
                assert_eq!(bv.value(), 0);
                assert_eq!(bv.len(), size_of::<$type>() * 8);
            }

            #[test]
            fn [<test_nanobv_ones_ $type>]() {
                let bv = NanoBV::<$type>::ones(size_of::<$type>() * 8);
                assert_eq!(bv.value(), $type::MAX);
                assert_eq!(bv.len(), size_of::<$type>() * 8);
            }
        }
        };
    }

    ImplNanoBVTest!(for u8, u16, u32, u64);
}
