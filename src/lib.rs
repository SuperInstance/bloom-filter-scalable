//! Scalable Bloom Filter that grows as needed

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const INITIAL_CAPACITY: usize = 1024;
const GROWTH_FACTOR: usize = 2;
const DEFAULT_ERROR_RATE: f64 = 0.01;

struct BloomSlice {
    bits: Vec<u64>,
    k: usize,
    capacity: usize,
    count: usize,
}

impl BloomSlice {
    fn new(capacity: usize, error_rate: f64) -> Self {
        let num_bits = ((-(capacity as f64) * error_rate.ln()) / (2.0_f64.ln().powi(2))).ceil() as usize;
        let num_bits = num_bits.max(64);
        let k = ((num_bits as f64 / capacity as f64) * 2.0_f64.ln()).ceil() as usize;
        let k = k.max(1).min(32);
        let words = (num_bits + 63) / 64;
        Self {
            bits: vec![0u64; words],
            k,
            capacity,
            count: 0,
        }
    }

    fn hash_at(&self, item: &impl Hash, i: usize) -> usize {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        (hasher.finish() as usize).wrapping_add(i * 0x9e3779b97f4a7c15_usize)
    }

    fn insert(&mut self, item: &impl Hash) -> bool {
        let mut was_new = false;
        for i in 0..self.k {
            let pos = self.hash_at(item, i) % (self.bits.len() * 64);
            let word = pos / 64;
            let bit = pos % 64;
            if self.bits[word] & (1u64 << bit) == 0 {
                was_new = true;
                self.bits[word] |= 1u64 << bit;
            }
        }
        if was_new {
            self.count += 1;
        }
        was_new
    }

    fn contains(&self, item: &impl Hash) -> bool {
        for i in 0..self.k {
            let pos = self.hash_at(item, i) % (self.bits.len() * 64);
            let word = pos / 64;
            let bit = pos % 64;
            if self.bits[word] & (1u64 << bit) == 0 {
                return false;
            }
        }
        true
    }

    fn is_full(&self) -> bool {
        self.count >= self.capacity
    }
}

/// A scalable bloom filter that adds slices when capacity is reached.
pub struct ScalableBloomFilter {
    slices: Vec<BloomSlice>,
    error_rate: f64,
    tightening_ratio: f64,
}

impl ScalableBloomFilter {
    pub fn new(error_rate: f64) -> Self {
        let first = BloomSlice::new(INITIAL_CAPACITY, error_rate);
        Self {
            slices: vec![first],
            error_rate,
            tightening_ratio: 0.85,
        }
    }

    pub fn insert(&mut self, item: &impl Hash) {
        if self.slices.last().unwrap().is_full() {
            let cap = self.slices.last().unwrap().capacity * GROWTH_FACTOR;
            let er = self.error_rate * self.tightening_ratio;
            self.slices.push(BloomSlice::new(cap, er));
        }
        self.slices.last_mut().unwrap().insert(item);
    }

    pub fn contains(&self, item: &impl Hash) -> bool {
        self.slices.iter().any(|s| s.contains(item))
    }

    pub fn slice_count(&self) -> usize {
        self.slices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut bf = ScalableBloomFilter::new(0.01);
        bf.insert(&"hello");
        bf.insert(&42);
        assert!(bf.contains(&"hello"));
        assert!(bf.contains(&42));
        assert!(!bf.contains(&"missing"));
    }
}
