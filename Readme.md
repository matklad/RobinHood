# Roobin Hood hashing micro benchmark

This is a super tiny benchmark of plain linear probing vs Robing Hood hashing.

## How to run

Download Rust via http://www.rustup.rs/. You need a nightly compiler and linux because
the benchmark uses linux specific performance counters. To use nightly compiler, issue
`rustup override add nightly`.

Run `cargo run --release` for plain linear probing.
Run `cargo run --release --features "robin-hood"` for Robin Hood hashing.

## Description

A hash table with fixed size of `8_388_608` is created and filled up to 9/10 of capacity.

The elements are random `u64`, hash function is identity.

The mean and variance of probe lengths are measured, as well as total duration.

## Results

```
$ rustc --version                                                                                                                                    ~/projects/hash
rustc 1.15.0-nightly (3bf2be9ce 2016-11-22)
```

Results for linear probing:

```
table capacity  8388608

probe mean      5.484624186744271
probe variance  17.441575475029587

duration        602ms
cache misses    35098829
```

Results for Robin Hood

```
table capacity  8388608

probe mean      5.498851021100442
probe variance  4.800754572809232

duration        861ms
cache misses    47173081
```

So, the mean probe length is the same, which is expected. The variance for Robin Hood is lower,
but not as low as one might expect. However, the actual runtime is bigger :( And the number of
cache misses is bigger as well by roughly the same proportion!
