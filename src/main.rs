extern crate rand;
extern crate perfcnt;

use std::collections::HashSet;
use std::time::Instant;
use rand::Rng;
use perfcnt::AbstractPerfCounter;

#[derive(Clone, Copy)]
struct Entry<T> {
    key: T,
    hash: usize,
}

struct Table<T> {
    mask: usize,
    hashes: Vec<usize>,
    keys: Vec<T>,
//    entries: Vec<Entry<T>>,
}

trait SimpleHash {
    fn hash(&self) -> usize;
}

impl SimpleHash for u64 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

impl<T> Table<T> where T: Default + SimpleHash + Copy + Eq {
    pub fn new(log_capacity: usize) -> Self {
        let n = 1 << log_capacity;
        Table {
            mask: n - 1,
            hashes: vec![0; n],
            keys: vec![T::default(); n],
//            entries: vec![Entry { key: T::default(), hash: 0 }; n],
        }
    }

    #[cfg_attr(not(feature = "robin-hood"), allow(unused))]
    pub fn insert(&mut self, mut key: T) {
        let mut hash = Table::hash_key(key);
        let mut pos = hash & self.mask;
        let mut dist = 0;
        loop {
            let entry_hash = unsafe { self.hashes.get_unchecked_mut(pos) };
            if *entry_hash == 0 {
                *entry_hash = hash;
                let entry_key = unsafe { self.keys.get_unchecked_mut(pos) };
                *entry_key = key;
                return
            }
            // assume no duplicated entries
            // debug_assert!(entry.key != key);

            #[cfg(feature = "robin-hood")]
            {
                let existing_key_dist = (pos + (self.mask + 1) - *entry_hash) & self.mask;
                if existing_key_dist < dist {
                    let entry_key = unsafe { self.keys.get_unchecked_mut(pos) };
                    std::mem::swap(&mut key, entry_key);
                    std::mem::swap(&mut hash, entry_hash);
                    dist = existing_key_dist;
                }
            }
            pos = (pos + 1) & self.mask;
            dist += 1;
        }
    }

    pub fn probe_len(&self, key: T) -> usize {
        let hash = Table::hash_key(key);
        let mut pos = hash & self.mask;
        let mut probes = 1;
        loop {
            let entry_hash = unsafe { self.hashes.get_unchecked(pos) };
            if *entry_hash == hash {
                let entry_key = unsafe { self.keys.get_unchecked(pos) };
                if *entry_key == key {
                    return probes;
                }
            }

            pos = (pos + 1) & self.mask;
            probes += 1;
        }
    }

    // E((x - Ex)^2) = E(x^2) -2xEx  +E(x)^2

    fn hash_key(key: T) -> usize {
        let h = key.hash();
        if h == 0 { 1 } else { h }
    }
}

fn rand_vec(n: usize) -> Vec<u64> {
    let mut set = HashSet::<u64>::with_capacity(n);
    while set.len() < n {
        set.insert(rand::random());
    }
    set.into_iter().collect()
}

fn main() {
    let log_cap = 23;
    let n = (1 << log_cap) * 90 / 100;
    let to_insert = rand_vec(n);

    let mut to_lookup = to_insert.clone();
    rand::thread_rng().shuffle(&mut to_lookup);

    let mut table = Table::<u64>::new(log_cap);
    for &key in to_insert.iter() {
        table.insert(key);
    }
    //    let mut lookups = vec![0; n];

    let start = Instant::now();

    let mut pc =
    perfcnt::linux::PerfCounterBuilderLinux::from_hardware_event(
        perfcnt::linux::HardwareEventType::CacheMisses
    ).finish().expect("Can't build a counter");

    pc.start().expect("Can not start the counter");
    let mut sum = 0;
    let mut sum_squares = 0;
    let mut number = 0;
    for &key in to_lookup.iter() {
        let p = table.probe_len(key);
        sum += p;
        sum_squares += p * p;
        number += 1;
    }
    pc.stop().expect("Can not start the counter");
    let cache_misses = pc.read().expect("Can not read the counter");

    let end = Instant::now();

    let mean = sum as f64 / number as f64;
    let variance = (sum_squares as f64 / number as f64 - mean * mean).sqrt();

    let duration = end - start;
    let duration_ms = duration.as_secs() as u32 * 1000 + duration.subsec_nanos() / 1_000_000;

    println!("table capacity  {}\n\n\
              probe mean      {}\n\
              probe variance  {}\n\n\
              duration        {}ms\n\
              cache misses    {}",
             (1 << log_cap),
             mean,
             variance,
             duration_ms,
             cache_misses);
}
