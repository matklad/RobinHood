extern crate rand;

use std::collections::HashSet;
use std::time::Instant;
use std::mem::swap;
use rand::Rng;

#[derive(Clone, Copy)]
struct Entry<T> {
    key: T,
    hash: usize,
}

struct Table<T> {
    mask: usize,
    entries: Vec<Entry<T>>,
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
            entries: vec![Entry { key: T::default(), hash: 0 }; n],
        }
    }

    pub fn insert(&mut self, mut key: T) {
        let mut hash = Table::hash_key(key);
        let mut pos = hash & self.mask;
        let mut dist = 0;
        loop {
            let entry = unsafe { self.entries.get_unchecked_mut(pos) };
            if entry.hash == 0 {
                entry.hash = hash;
                entry.key = key;
                return
            }
            // assume no duplicated entries
            debug_assert!(entry.key != key);

            // Robin Hood specific block. Comment out to get linear probing.
            {
                let existing_key_dist = (pos + (self.mask + 1) - entry.hash) & self.mask;
                if existing_key_dist < dist {
                    swap(&mut key, &mut entry.key);
                    swap(&mut hash, &mut entry.hash);
                    dist = existing_key_dist;
                }
            }
            pos = (pos + 1) & self.mask;
            dist += 1;
        }
    }

    pub fn contains(&self, key: T) -> Result<usize, usize> {
        let hash = Table::hash_key(key);
        let mut pos = hash & self.mask;
        let mut probes = 1;
        loop {
            let entry = unsafe { self.entries.get_unchecked(pos) };
            if entry.hash == 0 {
                return Result::Err(probes);
            }
            if entry.hash == hash && entry.key == key {
                return Result::Ok(probes);
            }
            pos = (pos + 1) & self.mask;
            probes += 1;
        }
    }

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
    let log_cap = 22;
    let n = (1 << log_cap) * 95 / 100;
    let to_insert = rand_vec(n);

    let mut to_lookup = to_insert.clone();
    rand::thread_rng().shuffle(&mut to_lookup);

    let mut table = Table::<u64>::new(log_cap);
    for &key in to_insert.iter() {
        table.insert(key);
    }
    let mut lookups = vec![0; n];

    let start = Instant::now();
    for (&key, l) in to_lookup.iter().zip(lookups.iter_mut()) {
        *l = match table.contains(key) {
            Ok(x) => x,
            Err(_) => unreachable!()
        };
    }
    let end = Instant::now();

    let mean = lookups.iter()
        .map(|&x| x as f64)
        .sum::<f64>() / n as f64;

    let variance = (lookups.iter()
        .map(|&x| x as f64)
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / (n - 1) as f64).sqrt();

    let duration = end - start;
    let duration_ms = duration.as_secs() * 1000 + duration.subsec_nanos() / 1_000_000;

    println!("capacity {}\n\
              mean     {}\n\
              variance {}\n\
              duration {}ms",
             (1 << log_cap),
             mean, variance,
             duration_ms);
}
