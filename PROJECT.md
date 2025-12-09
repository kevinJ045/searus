# SEARUS — PROJECT.md

> A step-by-step project plan and specification for Searus — a plugin-driven, multi-modal centralized search engine for Rust. This document is written to be clear and actionable for humans and other AI models that will help implement, test, and extend the project.

---

## Table of contents

1. Overview
2. Goals & non-goals
3. High-level architecture
4. Core concepts & data types
5. Searcher trait and plugin model
6. Query & Rules DSL
7. Scoring, normalization & ranking
8. Searcher implementations (initial set)
9. Indexing, storage & retrieval
10. Performance: ANN, caching, and batching
11. Image handling & multimodal embeddings
12. API surface (library + CLI)
13. Configuration & persistence
14. Testing, benchmarks & quality gates
15. CI / CD and releases
16. Roadmap / milestones (step-by-step)
17. Contribution guide & coding conventions
18. Appendix: Example flows & minimal call signatures

---

## 1. Overview

Searus is a Rust library and lightweight service that allows indexing and querying arbitrary user-defined entities with multiple search strategies (semantic text, vector, tags, image, etc.). The engine is plugin-first: each search method is a `Searcher` plugin which returns matches with a score and detail metadata. The engine merges and normalizes results into a unified ranking.

This document lays out the functional design, implementation steps, and a prioritized roadmap so developers (and AI assistants) can pick a clear next task and produce code that integrates well with the rest of the project.

---

## 2. Goals & non-goals

### Goals

* Provide a simple, ergonomic Rust API to search `Vec<T>` and other storage backends.
* Plugin architecture for easy addition of new search methods.
* Hybrid search: combine multiple searchers' scores into a single ranked result.
* Support text, vector (embeddings), tag, and image-based searches initially.
* Be easy to test, benchmark, and extend.
* Keep core engine synchronous and lightweight; allow async plugin implementations.

### Non-goals (initial)

* Full distributed cluster orchestration (defer to future milestone).
* Replacing production systems like Elasticsearch or FAISS immediately.
* Providing a production-grade disk-backed index for extremely large corpora (we will provide guidance and a minimal on-disk mode later).

---

## 3. High-level architecture

1. **Core engine (crate: `searus_core`)** — engine, result merging, scoring & normalization logic, Searcher trait, rules & query types.
2. **Searchers (crate: `searus_searchers`)** — implementations that provide specific search capabilities: semantic, vector, tags, image, fuzzy, etc. Each searcher implements the Searcher trait.
3. **Embeddings provider (crate: `searus_embeddings`)** — abstraction over embedding providers (local model, remote API). Returns `Vec<f32>` vectors.
4. **Index & storage adapters (crate: `searus_index`)** — in-memory indices, optional on-disk, simple APIs to store precomputed vectors, tags, and metadata.
5. **CLI & server (crate: `searus_cli`, `searus_server`)** — optional: a small service and CLI for quick experimentation.
6. **Examples & tests (crate: `searus_examples`)** — runnable examples demonstrating typical use-cases.

Modules communicate via typed Rust interfaces and share common data types in a `prelude` module for ergonomics.

---

## 4. Core concepts & data types

This section lists the primary types and short descriptions. Exact Rust structs shown in minimal pseudocode for clarity.

### 4.1 Primary types

SearusMatch<T>

* `item: T` or `id: EntityId` (depending on storage mode)
* `score: f32` (total normalized score)
* `field_scores: HashMap<String, f32>` (optional per-field breakdown)
* `details: Vec<SearchDetail>` (searcher-specific metadata)

SearchDetail

* enum describing the origin and metadata of the match, e.g. `Semantic{matched_terms, weight}`, `Vector{distance}`, `Tag{matched_tags}`

Searcher<T>

* trait plugin that performs search over a slice/iterator or an index and returns `Vec<SearchMatch<T>>` or `Vec<SearchPartial<T>>`.

Query

* `text: Option<String>`
* `vector: Option<Vec<f32>>`
* `tags: Option<Vec<String>>`
* `image: Option<ImageData>`
* `filters: Option<FilterExpr>`
* `options: SearchOptions`

SemanticRules

* DSL describing which fields to match, priority, matching strategy (Exact, Fuzzy, BM25), nested object rules.

SearchOptions

* `skip`, `limit`, `timeout_ms`, `weights: HashMap<SearcherKind, f32>`

EntityId

* `String` or `Uuid` depending on user preference (allow generic `K: Eq + Hash + Clone`).

---

## 5. Searcher trait and plugin model

Design the main trait to be minimal but flexible. Support both in-memory search over provided `Vec<T>` (useful for examples) and index-backed searchers.

### Minimal synchronous trait (core)

```rust
pub trait Searcher<T> {
    /// Returns matches for the given query. Implementations may ignore unsupported fields.
    fn search(&self, query: &Query, items: &[T]) -> Vec<SearchMatch<T>>;

    /// Optionally: search against an index/adapter
    fn search_index(&self, query: &Query, index: &dyn IndexAdapter) -> Vec<SearchMatch<T>> { vec![] }

    fn kind(&self) -> SearcherKind;
}
```

`SearcherKind` = enum { Semantic, Vector, Tags, Image, Fuzzy, Range, Geospatial, Custom(String) }

### Async support

Some searchers (like remote embeddings or remote ANN) are naturally async. Provide an optional async trait behind a cargo feature flag or let the implementer spawn tasks internally; the core engine can accept `Box<dyn AsyncSearcher>` when compiled with `async` feature.

### Plugin registration

SearusEngine will hold `Vec<Box<dyn Searcher<T>>>` with configurable weights. Provide a builder API for adding searchers:

```rust
let engine = SearusEngine::builder()
    .with(Box::new(SemanticSearch::new()))
    .with(Box::new(VectorSearch::new(index)))
    .build();
```

---

## 6. Query & Rules DSL

Provide two complementary ways to express queries:

1. **Programmatic Rust API** (preferred for library users)
2. **JSON/YAML** for remote configs

### Example programmatic API (goal)

```rust
let query = Query::builder()
    .text("how to borrow a book")
    .tags(vec!["rust", "search"])
    .options(SearchOptions::default().skip(10).limit(50))
    .build();

let rules = SemanticRules::builder()
    .field("title", FieldRule::default().matcher(Matcher::BM25).priority(2))
    .field("content", FieldRule::default().matcher(Matcher::Tokenized).priority(1))
    .object("user", ObjectRule::direct().field("username", FieldRule::exact().priority(3)))
    .build();
```

### Filters & boolean expressions

A `FilterExpr` AST supports simple field filters and boolean ops. Example:

* `views >= 1000 AND (tags CONTAINS "rust" OR published = true)`

Keep the filter AST small and easy to evaluate over in-memory items.

---

## 7. Scoring, normalization & ranking

Searchers produce raw scores or distances. The engine must normalize these so they can be blended.

### Normalization methods (implement at least two)

* **Min-Max**: `norm = (score - min) / (max - min)`
* **Softmax**: convert raw scores to probabilities
* **Inverse distance**: for distances from vectors: `1 / (1 + dist)`

### Final scoring

Final score for an item is a weighted sum:

```
final_score = Σ (normalized_score_i * weight_i)
```

Weights come from `SearchOptions::weights` or per-searcher configuration. Provide defaults (e.g., semantic 0.6, vector 0.4).

### Field-level contribution

Field rules in `SemanticRules` should produce field-level scores that are combined by that searcher into a searcher-level score. Include `field_scores` in `SearusMatch` for explainability.

---

## 8. Searcher implementations (initial set)

Prioritize building the following searchers in this order. Each entry includes the minimal interface, data it needs, and a recommended incremental implementation strategy.

### 8.1 SemanticSearch (text-based)

* **Purpose**: rule-based text matching using BM25, tokenization, fuzzy matching.
* **Inputs**: `text` field from `Query`, `SemanticRules`, items' field values.
* **Implementation steps**:

  1. Implement tokenizers and basic term frequency counters.
  2. Implement a basic BM25 scorer for small corpora.
  3. Add fuzzy matching fallback using `strsim` or Levenshtein distance for short fields.
  4. Produce `SearchDetail::Semantic { matched_terms, field_score }`.

### 8.2 TaggedSearch

* **Purpose**: match items by tags with boost/priorities.
* **Inputs**: `query.tags`, each item's tag list.
* **Implementation steps**: exact matches, partial matches (prefix), and weighted counts.

### 8.3 VectorSearch (embeddings + ANN)

* **Purpose**: find nearest vectors for `query.vector` or text->vector using embeddings provider.
* **Inputs**: `query.vector` or precomputed item vectors stored in an index.
* **Implementation steps**:

  1. Build an in-memory brute-force search first.
  2. Add HNSW approximate NN via an existing Rust crate or WASM-wrapped lib (optional). Document how to plug FAISS or Annoy externally.
  3. Implement distance→score conversion and normalization.

### 8.4 ImageSearch & ImageSemanticSearch

* **Purpose**: reverse image search (image->image) and image semantic (image↔text) using CLIP-like embeddings.
* **Inputs**: `query.image`, precomputed image embeddings, CLIP text embeddings.
* **Implementation steps**:

  1. Standardize `ImageData` type (bytes + optional mime).
  2. Use an embeddings provider that supports images (local script or remote API) or stub with random vectors for tests.
  3. Reuse the VectorSearch path for nearest-neighbor lookups.

### 8.5 FuzzySearch

* **Purpose**: catch typos using Damerau-Levenshtein or trigram similarity.
* **Implementation steps**: integrate `strsim` or a trigram index for larger corpora.

### 8.6 Range, Geospatial, Boolean

* **Purpose**: filters and domain-specific comparisons.
* **Implementation**: simple evaluators over item fields; geospatial uses Haversine distance.

---

## 9. Indexing, storage & retrieval

Start with a simple in-memory index for development and testing. Plan an adapter trait for pluggable index backends.

### 9.1 IndexAdapter trait

* `fn put(&mut self, id: EntityId, metadata: &T, vectors: Option<Vec<f32>>, tags: Option<Vec<String>>) -> Result<()>;`
* `fn remove(&mut self, id: &EntityId) -> Result<()>;`
* `fn get(&self, id: &EntityId) -> Option<T>;`
* `fn knn(&self, vector: &[f32], k: usize) -> Vec<(EntityId, f32)>;` (for vector search)

Implementations:

* `InMemIndex` (HashMaps + Vec storage)
* `FileIndex` (simple on-disk using mmap or binary format) — milestone 2
* docs for how to integrate FAISS/Annoy/other external tools — milestone 3

---

## 10. Performance: ANN, caching, and batching

* Start brute-force for correctness.
* Add HNSW-based ANN for speed: use a Rust crate (research current crates and document tradeoffs).
* Batch embedding resolution when generating vectors for many items (reduce API calls).
* Cache embeddings and ANN results for repeated queries.
* Provide concurrency options: single-threaded for deterministic tests, rayon parallel map for speed.

---

## 11. Image handling & multimodal embeddings

* Define `ImageData` (bytes, width, height, mime).
* Provide a small `ImageEmbeddingsProvider` trait mirroring the text embeddings provider.
* Document using CLIP/OpenCLIP/remote APIs. For offline dev, provide a simple utility to compute color histograms and perceptual hashes.
* For cross-modal (text↔image) use CLIP-like joint embedding space.

---

## 12. API surface (library + CLI)

### Library (Rust crate)

* `SearusEngine<T>` builder & `search` method.
* `Searcher<T>` trait and built-in searchers.
* `IndexAdapter` trait and `InMemIndex`.
* Types: `Query`, `SearchOptions`, `SemanticRules`, `SearusMatch<T>`, `SearchDetail`.

Minimal example usage (pseudocode in rust):

`(see examples folder for working code)`

### CLI / Server

* CLI to index JSON/CSV files and run example queries.
* Optional small HTTP server exposing a JSON API for querying (unpinned for milestone 3).

---

## 13. Configuration & persistence

* Keep a `Config` struct for default weights, searcher settings, and index paths.
* Support JSON/YAML config files.
* For persistence, serialize `InMemIndex` into a compact binary or JSON (for small corpora).

---

## 14. Testing, benchmarks & quality gates

* Unit tests for each searcher with synthetic datasets.
* Integration tests: combine multiple searchers and verify ranking/normalization invariants.
* Benchmarks: use `criterion` for measuring end-to-end query latency and ANN performance.
* Add CI steps to run `cargo test` and basic benchmarks on push.

---

## 15. CI / CD and releases

* GitHub Actions pipeline:

  1. Run `cargo fmt`, `cargo clippy`, `cargo test`.
  2. Build docs (`cargo doc --no-deps`), run benchmarks optionally.
  3. On tag, build release artifacts and publish the crate.

* Use semantic versioning (semver).

---

## 16. Roadmap / milestones (step-by-step)

### Milestone 0 — Project init (1–2 days)

* Repo scaffold, crates layout, `README.md`, `PROJECT.md` (this file).
* Add `searus_core` crate with base types and the `Searcher` trait.
* Add `searus_examples` with a minimal `Post` example and an in-memory `SearusEngine`.

Deliverables:

* `SearusEngine::new()` skeleton
* Minimal `search` that returns items unchanged

### Milestone 1 — Semantic search & rules DSL (3–7 days)

* Implement the `SemanticRules` builder API and parser.
* Implement a tokenization pipeline and BM25 scorer for small corpora.
* Implement `SemanticSearch` searcher that runs over `Vec<T>` using reflection/serde for field access or explicit accessor closures.
* Unit tests and example.

Deliverables:

* `SemanticSearch` implemented, docs and example run.

### Milestone 2 — Tagged & Fuzzy search + ranking merge (2–4 days)

* Implement `TaggedSearch` and `FuzzySearch`.
* Implement normalization & weighted merging in the core engine.
* Provide `SearchOptions` for per-searcher weights.

Deliverables:

* Multi-search merging with configurable weights, example and tests.

### Milestone 3 — Embeddings & VectorSearch (4–10 days)

* Add `searus_embeddings` abstraction and a local stub provider.
* Add `InMemIndex` storing vectors and brute-force KNN.
* Implement `VectorSearch` and integrate score normalization.

Deliverables:

* Vector search works end-to-end with stub embeddings.

### Milestone 4 — ANN & Performance (4–7 days)

* Integrate or document HNSW/FAISS usage.
* Add caching, batching, and bench tests.

Deliverables:

* ANN-backed `VectorSearch` with performance benchmarks.

### Milestone 5 — Image & Multimodal (4–8 days)

* Implement `ImageData` handling and `ImageSearch` using embeddings.
* Implement cross-modal (text↔image) using CLIP embeddings.

Deliverables:

* Image search example and tests.

### Milestone 6 — CLI / Server / Packaging (3–6 days)

* Implement CLI for indexing and querying local datasets.
* Optional HTTP server for experimentation.

Deliverables:

* CLI and small server prototype.

### Milestone 7 — Polish & docs (ongoing)

* Documentation, examples, API docs, contribution guide, and publications.

---

## 17. Contribution guide & coding conventions

* Follow Rust idioms. Use `rustfmt` and `clippy` rules.
* Keep functions small and single-purpose.
* Prefer composition over macros, but provide a macro-based DSL later for ergonomics.
* Use `serde` for flexible data ingestion where appropriate.
* Tests: aim for deterministic unit tests; for probabilistic tests (ANN) use seeded RNGs.

Repository conventions:

* `crates/` with `searus_core`, `searus_searchers`, `searus_index`, `searus_embeddings`, `searus_cli`, `searus_examples`.
* `examples/` folder run by CI for smoke tests.

---

## 18. Appendix: Example flows & minimal call signatures

### 18.1 Minimal in-memory flow (what to implement first)

1. Create `SearusEngine` and add `SemanticSearch`:

   * `let engine = SearusEngine::new().with(SemanticSearch::new());`
2. Build `SemanticRules` for `Post` fields.
3. Call `engine.search(posts, rules, query, SearchOptions::default())`.
4. Engine runs each registered searcher over the supplied `Vec<T>`, collects raw matches, normalizes scores, merges matches by item id, sorts by final score and returns `Vec<SearusMatch<T>>`.

### 18.2 Minimal trait signatures

```rust
pub struct Query { ... }
pub struct SearchOptions { pub skip: usize, pub limit: usize, pub weights: HashMap<SearcherKind, f32> }

pub struct SearusMatch<T> {
    pub item: T,
    pub score: f32,
    pub field_scores: HashMap<String, f32>,
    pub details: Vec<SearchDetail>,
}

pub trait Searcher<T>: Send + Sync {
    fn kind(&self) -> SearcherKind;
    fn search(&self, query: &Query, items: &[T]) -> Vec<SearusMatch<T>>;
}
```
