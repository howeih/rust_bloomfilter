bloom filter written in Rust
=================

### Installation:
Cargo.toml:
```
    bloomfilter = { git = "https://github.com/howeih/rust_bloomfilter.git", branch = "master" }
```

### Usage :
```
    extern crate bloomfilter;

    let mut bloom = bloomfilter::Bloom::new(100,0.001);
    let item = vec![1u8,2u8,3u8];
    bloom.set(&item);
    assert!(bloom.check(&item) == true);
```

- Bloom::new(m,n) where<br/>
m is the number of items in the filter<br/>
n is the probability of false positives, float between 0 and 1
