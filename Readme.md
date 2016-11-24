# Roobin Hood hashing micro benchmark

This is a super tiny benchmark of plain linear probing vs Robing Hood hashing.

## How to run

Download Rust via http://www.rustup.rs/.

Run `cargo run --release`.

To switch between linear and Robin Hood, comment a marked piece of code in `src/main.rs`

## Description

A hash table with fixed size of `8_388_608` is created and filled up to 9/10 of capacity.

The elements are random `u64`, hash function is identity.

The mean and variance of probe lengths are measured, as well as total duration.

## Results

Results for linear probing:

```
capacity 8388608
mean len 5.494383719083567
variance 17.50909404990507
duration 660ms
```

Results for Robin Hood

```
capacity 8388608
mean len 5.496199938885369
variance 4.8286259257836015
duration 901ms
```

So, the mean probe length is the same, which is expected. The variance for Robin Hood is lower,
but not as low as one might expect. However, the actual runtime is bigger :(