# SEARUS — OPTIMIZATION.md

> A comprehensive guide to optimizing the Searus search engine for speed and efficiency, specifically for medium to large datasets (e.g., 100k+ entities).

---

## Table of contents

1. Overview
2. Identifying bottlenecks
3. Precomputation & caching
4. Parallelism & chunking strategies
5. Approximate methods (ANN / HNSW)
6. Early filtering & pruning
7. SIMD & optimized string operations
8. Incremental scoring & early cutoff
9. Indexing & secondary indices
10. Memory & allocation optimization
11. Benchmarking & profiling
12. Step-by-step optimization plan
13. Roadmap / future improvements

---

## 1. Overview

This document provides **strategies and concrete steps** to optimize the Searus search engine for faster queries. It covers both CPU-bound operations (semantic, fuzzy search) and vector-based searches. The goal is to reduce query time for 100k+ entities from seconds to hundreds of milliseconds, while maintaining correctness and flexibility.

---

## 2. Identifying bottlenecks

Typical sources of slowness:

* Recomputing tokenized or preprocessed fields every query
* Brute-force vector distance calculations
* Iterating over all entities for every search
* String allocations and cloning inside loops
* Mutex/RwLock contention in parallel code

**Tools to measure:**

* `std::time::Instant` for coarse timing
* `cargo bench` and `criterion` for benchmarks
* `perf` / `valgrind` / `cargo-flamegraph` for CPU profiling

---

## 3. Precomputation & caching

* Pre-tokenize all text fields and store them inside each entity or in a separate `SearchIndex` struct.
* Precompute embeddings for text and images where applicable.
* Pre-lowercase, trim, or normalize strings for matching.
* Cache field hash maps, trigram sets, or other indices to avoid recomputation per query.

Example:

```rust
struct PostSearchData {
    title_tokens: Vec<String>,
    content_tokens: Vec<String>,
    username_lc: String,
    tags: Vec<String>,
    vector: Vec<f32>,
}
```

---

## 4. Parallelism & chunking strategies

* Use `rayon::par_iter()` or `tokio` tasks for async searchers.
* Split entity list into chunks that fit CPU cache for better utilization.
* Avoid shared mutable state inside parallel loops to reduce contention.
* Combine results after parallel computation rather than locking during iteration.

---

## 5. Approximate methods (ANN / HNSW)

* Replace brute-force vector distance search with **Approximate Nearest Neighbor (ANN)** algorithms.
* Options: `hnsw_rs`, `annoy`, `nmslib` (Rust bindings).
* Query time drops from linear O(N) to logarithmic or near-constant for typical sizes.
* For semantic vector search, precompute vectors and build ANN index once.

---

## 6. Early filtering & pruning

* Apply cheap filters first: tags, numeric fields, exact matches.
* Only pass filtered candidates to expensive scoring routines (semantic or vector).
* Example:

```text
Filter: tags CONTAINS 'filler' AND views >= 1000
Then: fuzzy / semantic scoring only on filtered subset
```

* This reduces the number of entities each searcher must process, improving performance dramatically.

---

## 7. SIMD & optimized string operations

* For semantic/fuzzy matching:

  * Use SIMD-accelerated string comparison libraries (e.g., `simd-str`, `fastrun`).
  * Avoid allocating new strings inside loops; work with `&str` slices.
  * Use n-gram indices or trigrams to quickly shortlist candidates.

---

## 8. Incremental scoring & early cutoff

* Score fields in order of priority.
* Maintain a current top-N threshold and skip lower-priority fields if maximum achievable score is below the threshold.
* Reduces computation for long posts or low-priority matches.

---

## 9. Indexing & secondary indices

* Maintain secondary indices to avoid scanning all entities:

  * Tag → Vec<EntityId>
  * Lowercased token → Vec<EntityId>
  * Precomputed vectors in ANN index
* Query process:

  1. Fetch candidate IDs from secondary indices
  2. Compute expensive scoring only on this subset
* This is especially effective when only a small subset of entities match the filters.

---

## 10. Memory & allocation optimization

* Pre-allocate `Vec` capacity where possible
* Reuse static strings (`Arc<String>`) for repeated content
* Avoid cloning heavy objects in inner loops
* Use stack-allocated small vectors (`smallvec`) when appropriate
* Reduce heap allocations in high-frequency code paths

---

## 11. Benchmarking & profiling

* Always measure before and after optimizations.
* Use representative datasets (100k+ entities) for realistic benchmarks.
* Track:

  * Total query time
  * Per-searcher contribution
  * Memory usage / allocations
* Identify hot loops and optimize them first.

---

## 12. Step-by-step optimization plan

1. **Precompute fields & embeddings**

   * Tokenize text, lowercase strings, precompute vectors
2. **Apply early filters**

   * Tags, numeric fields, exact matches
3. **Implement parallel search**

   * Use `rayon::par_iter` with chunking
4. **Incremental scoring / early cutoff**

   * Skip low-potential entities early
5. **Introduce ANN for vector search**

   * HNSW / FAISS / Annoy integration
6. **Optimize string operations**

   * SIMD / n-gram / trigram indices
7. **Profile & tweak chunk sizes / thread pool**
8. **Benchmark end-to-end query times**

> Goal: reduce query time from 4s to <500ms for 100k entities

---

## 13. Roadmap / future improvements

* **Dynamic caching of frequently queried results**
* **Persistent ANN indices on disk**
* **Sharded / partitioned search for >1M entities**
* **Optional GPU acceleration for embedding computation**
* **Hybrid top-k search combining semantic, vector, and tag scores efficiently**
* **Adaptive thresholding based on query complexity**

---

**Summary:**

By precomputing data, applying early filters, using approximate nearest neighbor algorithms, and optimizing parallel and string operations, Searus can reduce search times dramatically for medium-to-large datasets while keeping the API simple and extensible. This roadmap provides concrete steps for incremental performance gains while maintaining correctness and flexibility.
