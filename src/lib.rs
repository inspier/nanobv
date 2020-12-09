use core::mem::size_of;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NanoBV<T = u32> {
    data: T,
    length: usize,
}

impl<T> NanoBV<T> {
    pub const fn new(data: T, length: usize) -> Self {
        NanoBV { data, length }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.length
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }
}

#[macro_use]
macro_rules! ImplNanoBVCommon {
    (for $($type:tt),+) => {
        $(ImplNanoBVCommon!($type);)*
    };

    ($type:ident) => {

        impl NanoBV<$type> {
            pub const fn clear(&self, begin: usize, end: usize) -> $type {
                if begin > end || (end - begin) > self.length {
                    self.data
                } else {
                    $type::MAX << end | ((1 << begin) - 1) & self.data
                }
            }

            pub const fn reverse(&self) -> NanoBV<$type> {
                let mut bit_size = (size_of::<$type>() * 8);
                let mut mask = !0;
                let mut v = self.data;
                bit_size >>= 1;
                while bit_size > 0 {
                    mask ^= (mask << bit_size);
                    v = ((v >> bit_size) & mask) | ((v << bit_size) & !mask);
                    bit_size >>= 1;
                }
                v >>= (size_of::<$type>() * 8) - self.length;
                NanoBV::<$type>::new(v, self.length)
            }
        }
    };
}

ImplNanoBVCommon!(for u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smallbitvec_u8_reverse() {
        assert_eq!(
            NanoBV::<u8>::new(0x1D, 7).reverse(),
            NanoBV::<u8>::new(0x5C, 7)
        );
    }

    #[test]
    fn smallbitvec_u16_reverse() {
        assert_eq!(
            NanoBV::<u16>::new(0x071F, 13).reverse(),
            NanoBV::<u16>::new(0x1F1C, 13)
        );
    }

    #[test]
    fn smallbitvec_u32_reverse() {
        assert_eq!(
            NanoBV::<u32>::new(0xC71F, 17).reverse(),
            NanoBV::<u32>::new(0x0001_F1C6, 17)
        );
    }

    #[test]
    fn smallbitvec_u64_reverse() {
        assert_eq!(
            NanoBV::<u64>::new(0xC71F, 17).reverse(),
            NanoBV::<u64>::new(0x0001_F1C6, 17)
        );
    }
}
