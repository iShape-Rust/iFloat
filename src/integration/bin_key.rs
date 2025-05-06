use crate::int::point::IntPoint;
use i_key_sort::bin_key::index::{BinKey, BinLayout};

impl BinKey<i32> for IntPoint {
    #[inline(always)]
    fn bin_key(&self) -> i32 {
        self.x
    }

    #[inline(always)]
    fn bin_index(&self, layout: &BinLayout<i32>) -> usize {
        layout.index(self.x)
    }
}
