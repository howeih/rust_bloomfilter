pub mod bloomfilter;

#[cfg(test)]
mod tests {
    use super::*;
    extern crate rand;
    use tests::rand::Rng;

    #[test]
    fn bloom_test_set() {
        let mut bloom = bloomfilter::Bloom::new(100, 0.001);
        let key: &Vec<u8> = &rand::thread_rng().gen_iter::<u8>().take(25).collect();
        debug_assert!(bloom.check(key) == false);
        bloom.set(&key);
        debug_assert!(bloom.check(key) == true);
    }

    #[test]
    fn bloom_test_clear() {
        let mut bloom = bloomfilter::Bloom::new(100, 0.001);
        let key: Vec<u8> = rand::thread_rng().gen_iter::<u8>().take(25).collect();
        bloom.set(&key);
        debug_assert!(bloom.check(&key) == true);
        bloom.clear();
        debug_assert!(bloom.check(&key) == false);
    }
}
