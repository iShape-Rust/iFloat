use i_key_sort::bin_key::index::{BinKey, BinLayout};
use crate::int::point::IntPoint;


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