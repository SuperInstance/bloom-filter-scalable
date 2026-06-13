# Bloom Filter Scalable

**A Rust library for auto-growing Bloom filters** — maintains a target false-positive rate as items are added indefinitely, without knowing the set cardinality in advance.

## Why It Matters

Standard Bloom filters require you to specify expected item count upfront. Exceed that capacity, and the false-positive rate spirals. In real-world systems — event streams, network packets, unique-key tracking — you often don't know the cardinality ahead of time.

The **Scalable Bloom Filter** (SBF), introduced by Almeida et al. (2007), solves this by adding new filter slices as needed. Each new slice has a larger capacity and a tighter false-positive rate, so the composite FP rate stays bounded.

SBFs are used in:
- **Network intrusion detection** — tracking seen signatures in unbounded traffic
- **Deduplication systems** — eliminating duplicate events in stream processing
- **Web crawlers** — tracking visited URLs without a fixed size limit
- **Distributed caches** — probabilistic membership tests across a cluster

## How It Works

**Slice architecture**: An SBF is a chain of `BloomSlice` filters. The first slice has capacity 1024 and the target FP rate (e.g., 1%). When it fills up, a new slice is created with double the capacity and a tighter FP rate (multiplied by a "tightening ratio" of 0.85).

**Compound false-positive rate**: If each slice *i* has FP rate *p · r^i* (where r is the tightening ratio), the composite FP rate is bounded by `p / (1 − r)`. With p = 0.01 and r = 0.85, the asymptotic FP rate is ~6.7%.

**Membership test**: Query ALL slices. If any slice returns positive, the item "might be" in the set. This is the union of all slice membership tests.

**Insertion**: Always insert into the last (current) slice. When its count reaches capacity, create a new slice.

**Hashing**: Each slice uses k hash functions (computed from the optimal formula `k = (m/n)·ln2`). The hash at index *i* is `h(x) + i · 0x9e3779b97f4a7c15` — a golden-ratio multiplier for double hashing.

## Quick Start

```rust
use bloom_filter_scalable::ScalableBloomFilter;

let mut bf = ScalableBloomFilter::new(0.01); // 1% target FP rate

// Insert items — the filter grows automatically
for i in 0..10_000 {
    bf.insert(&i);
}

assert!(bf.contains(&5000));
assert!(!bf.contains(&99_999)); // probably correct
println!("Slices used: {}", bf.slice_count());
```

## API

- **`ScalableBloomFilter`** — Auto-growing Bloom filter
  - `new(error_rate)` — Create with target FP rate
  - `insert(item)` — Add an item (grows if needed)
  - `contains(item)` → `bool` — Probabilistic membership test
  - `slice_count()` → `usize` — Number of internal filter slices

## Architecture Notes

Provides the unbounded-membership primitive for SuperInstance streaming pipelines. Used when tracking event deduplication keys across long time horizons where the total cardinality isn't predictable. See the [architecture overview](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## License

MIT
