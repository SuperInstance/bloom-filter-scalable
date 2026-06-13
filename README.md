# bloom-filter-scalable

A **Scalable Bloom Filter** implementation in pure Rust that dynamically grows its capacity by appending new filter slices as elements are inserted, maintaining a target false-positive rate across unbounded data streams.

## Why It Matters

Standard Bloom filters have a fixed capacity — once full, the false-positive rate degrades rapidly. Scalable Bloom Filters (SBF), introduced by Almeida et al. (2007), solve this by creating a series of geometrically-growing slices, each with a tightened error budget. This makes SBFs ideal for:

- **Unbounded data streams** — web crawlers, event logs, deduplication
- **Database engines** — Cassandra, HBase, and Postgres use Bloom filters for SSTable lookups
- **CDN and routing** — membership testing with bounded memory
- **Blockchain** — SPV clients and transaction bloom filters

## How It Works

### Slice Sizing

Each new slice `i` is sized for capacity $c_i = c_0 \cdot r^i$ where $r$ is the growth factor (2× in this implementation). The number of bits per slice follows the optimal Bloom filter formula:

$$m = \left\lceil \frac{-n \cdot \ln p}{(\ln 2)^2} \right\rceil$$

where $n$ is the expected capacity and $p$ is the target false-positive rate.

### Hash Functions

The number of hash functions $k$ per slice is:

$$k = \left\lceil \frac{m}{n} \cdot \ln 2 \right\rceil$$

This implementation uses double hashing: a single `DefaultHasher` evaluation combined with linear probing via the golden ratio constant `0x9e3779b97f4a7c15` to synthesize $k$ independent hash positions.

### Error Tightening

Each new slice tightens its error budget by the **tightening ratio** $r_t = 0.85$:

$$p_i = p_0 \cdot r_t^i$$

The compound false-positive probability across $s$ slices is:

$$P_{\text{total}} = 1 - \prod_{i=0}^{s-1}(1 - p_i)$$

Because the series $\sum p_0 \cdot r_t^i = p_0 / (1 - r_t)$ converges, the SBF achieves a finite asymptotic error bound.

### Big-O Complexity

| Operation | Time | Space |
|-----------|------|-------|
| `insert(x)` | O(k) | O(m / 64) words |
| `contains(x)` | O(k · s) worst, O(k) typical | — |
| Slice allocation | O(m) | O(m) |

Where k = hash functions per slice, s = number of slices, m = bits per slice.

## Quick Start

```rust
use bloom_filter_scalable::ScalableBloomFilter;

let mut bf = ScalableBloomFilter::new(0.01); // 1% target FPR
bf.insert(&"hello");
bf.insert(&42);

assert!(bf.contains(&"hello"));
assert!(bf.contains(&42));
assert!(!bf.contains(&"missing"));

println!("Slices: {}", bf.slice_count());
```

## API

| Method | Description |
|--------|-------------|
| `ScalableBloomFilter::new(error_rate: f64)` | Create with target FPR |
| `insert(&impl Hash)` | Insert an element |
| `contains(&impl Hash) → bool` | Membership query (may have false positives) |
| `slice_count() → usize` | Current number of filter slices |

## Architecture Notes

The **γ + η = C** link: each slice's bit array (γ) captures inserted elements, while the tightening ratio (η) ensures the compound error budget converges. Together they conserve the invariant C — the total false-positive rate stays below the configured threshold regardless of how many elements are inserted.

When the active slice reaches its capacity threshold, a new slice is allocated with 2× capacity and 0.85× the error budget. This geometric growth ensures amortized O(1) insertion cost.

## References

- Almeida, P. S., Baquero, C., Preguiça, N., & Hutchison, D. (2007). *Scalable Bloom Filters.* Information Processing Letters, 101(6), 255–261.
- Bloom, B. H. (1970). *Space/time trade-offs in hash coding with allowable errors.* Communications of the ACM, 13(7), 422–426.
- Mitzenmacher, M., & Upfal, E. (2017). *Probability and Computing: Randomization and Probabilistic Techniques in Algorithms and Data Analysis.* Cambridge University Press.
- Kirsch, A., & Mitzenmacher, M. (2006). *Less Hashing, Same Performance: Building a Better Bloom Filter.* ESA 2006.

## License

MIT
