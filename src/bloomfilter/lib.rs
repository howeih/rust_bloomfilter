#![crate_name = "bloomfilter"]
extern crate bit_vec;
extern crate num_cpus;
extern crate rand;

use std::sync::mpsc::channel;
use std::thread::JoinHandle;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use bit_vec::BitVec;
use std::f64;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::hash_map::DefaultHasher;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};


#[cfg(test)]
use rand::Rng;

pub struct Bloom {
    bitmap: BitVec,
    filter_size: usize,
    hash_size: usize,
    hash_functions: Vec<DefaultHasher>,
}

impl Bloom {
    fn optimize_filter_size(items_count: usize, false_positive_rate: f64) -> usize {
        let filter_size = -1.0 * ((items_count as f64) * false_positive_rate.ln()) /
            (f64::ln(2.0)).powi(2);
        filter_size.ceil() as usize
    }

    fn optimize_hash_size(filter_size: usize, items_count: usize) -> usize {
        let hash_size = (filter_size as f64) / (items_count as f64) * f64::ln(2.0);
        hash_size.ceil() as usize
    }

    fn init_func_vec(hash_size: usize) -> Vec<DefaultHasher> {
        let mut hash_functions: Vec<DefaultHasher> = Vec::with_capacity(hash_size);
        for _ in 0..hash_size {
            let s = RandomState::new();
            let hasher = s.build_hasher();
            hash_functions.push(hasher);
        }
        hash_functions
    }

    pub fn new(items_count: usize, false_positive_rate: f64) -> Self {
        assert!(items_count > 0 && false_positive_rate >= 0.0 && false_positive_rate <= 1.0);
        let filter_size = Self::optimize_filter_size(items_count, false_positive_rate);
        let hash_size = Self::optimize_hash_size(filter_size, items_count);
        let hash_functions = Self::init_func_vec(hash_size);
        Self {
            bitmap: BitVec::from_elem(filter_size, false),
            filter_size: filter_size,
            hash_size: hash_size,
            hash_functions: hash_functions,
        }
    }

    fn init_hasher_id_sender(&self, sender: Sender<usize>) -> JoinHandle<()> {
        let hash_size = self.hash_size;
        let t_supplier = thread::spawn(move || {
            let mut hash_id: Vec<usize> = (0..hash_size).collect();
            while hash_id.len() > 0 {
                let id = hash_id.pop().unwrap();
                sender.send(id).unwrap();
            }
        });
        t_supplier
    }

    fn calculate_hash(
        &self,
        mut join_handler: Vec<JoinHandle<()>>,
        receiver: Receiver<usize>,
        item: &Vec<u8>,
    ) -> Vec<u64> {
        let num_cpus = num_cpus::get();
        let rcver = Arc::new(Mutex::new(receiver));
        let hash_values = Arc::new(Mutex::new(Vec::<u64>::new()));
        for _ in 0..num_cpus {
            let recv = Arc::clone(&rcver);
            let mut hash_functions = self.hash_functions.clone();
            let local_hash_values = hash_values.clone();
            let item_clone = item.clone();
            let t_id = thread::spawn(move || loop {
                let recv = recv.lock().unwrap().recv();
                match recv {
                    Ok(hash_id) => {
                        let mut hasher = &mut hash_functions[hash_id];
                        hasher.write(&item_clone);
                        local_hash_values.lock().unwrap().push(hasher.finish());
                    }
                    _ => break,
                }
            });
            join_handler.push(t_id);
        }
        for join in join_handler {
            join.join().unwrap();
        }
        let mut hash_result = vec![];
        hash_result.extend(hash_values.lock().unwrap().iter());
        hash_result
    }

    fn bloom_hash(&self, item: &Vec<u8>) -> Vec<u64> {
        let (sender, receiver) = channel();
        let mut join_handler = vec![];
        let t_supplier = self.init_hasher_id_sender(sender);
        join_handler.push(t_supplier);
        self.calculate_hash(join_handler, receiver, item)
    }
    
    pub fn set(&mut self, item: &Vec<u8>) {
        let hash_values = self.bloom_hash(item);
        for hash in hash_values {
            let bit_offset = (hash % self.filter_size as u64) as usize;
            self.bitmap.set(bit_offset, true);
        }
    }

    pub fn check(&self,  item: &Vec<u8>) -> bool{
        let hash_values = self.bloom_hash(item);
        for hash in hash_values {
            let bit_offset = ( hash % self.filter_size as u64) as usize;
            if self.bitmap.get(bit_offset).unwrap() == false {
                return false;
            }
        }
        true
    }

    pub fn clear(&mut self) {
        self.bitmap.clear()
    }
}

#[test]
fn bloom_test_set() {
    let mut bloom = Bloom::new(100,0.001);
    let key: &Vec<u8> = &rand::thread_rng().gen_iter::<u8>().take(25).collect();
    debug_assert!(bloom.check(key) == false);
    bloom.set(&key);
    debug_assert!(bloom.check(key) == true);
}

#[test]
fn bloom_test_clear() {
    let mut bloom = Bloom::new(100,0.001);
    let key: Vec<u8> = rand::thread_rng().gen_iter::<u8>().take(25).collect();
    bloom.set(&key);
    debug_assert!(bloom.check(&key) == true);
    bloom.clear();
    debug_assert!(bloom.check(&key) == false);
}