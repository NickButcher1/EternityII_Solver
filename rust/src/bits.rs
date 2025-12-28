#[inline(always)]
pub fn set_bit(bits: &mut [u64; 5], index: usize) {
    bits[index >> 6] |= 1u64 << (index & 63);
}

#[inline(always)]
pub fn clear_bit(bits: &mut [u64; 5], index: usize) {
    bits[index >> 6] &= !(1u64 << (index & 63));
}

#[inline(always)]
pub fn is_clear(bits: &[u64; 5], index: usize) -> bool {
    (bits[index >> 6] & (1u64 << (index & 63))) == 0
}
