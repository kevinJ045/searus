# Custom Searchers

*A guide to implementing your own search strategies inside Searus.*

Custom searchers allow library users to plug in **any search algorithm** they want: BM25, TF-IDF variants, vector scoring, hybrid scoring, tree-based search, LSH, or experimental models. Searus exposes a simple trait-based interface so searchers can be composed and combined.

---

## 1. Searcher Trait

Every custom searcher implements the main `Searcher` trait.

### Trait Definition

A searcher takes a query and a read-only context, and returns a scored hit list.

### Goals of the Trait

* Searchers must be **pure** (no mutation of global state).
* They must be **thread-safe** (`Send + Sync`).
* They should run **independently** so the engine can call multiple searchers in parallel.

---

## 2. Registering Custom Searchers

Users attach searchers to the engine during configuration.

Examples:

* Only TF-IDF
* BM25 + Vector Search
* Hybrid weighted ranking with custom logic

---

## 3. Calling Specific Searchers

Searchers can be invoked individually (e.g. for testing) or through the main dispatcher that merges results.

The user can:

* Call a named searcher directly.
* Let the engine run all enabled searchers then merge results.

---

## 4. SearchContext

The context gives searchers **read-only access** to:

* Inverted index
* Document frequency & statistics
* Global metadata
* Vector index (optional)
* Custom indexes future searchers may add

The context is designed to be:

* Cheap to clone (ref-only)
* Stable for the lifetime of the index
* Safe to access concurrently

---

## 5. Example: TF-IDF Searcher

This illustrates a minimal, fast custom searcher implementation.

### Pattern

* For each query token, grab posting lists.
* Compute TF × IDF.
* Combine scores per doc.
* Return sorted scored documents.

This pattern is recommended for simple text searchers.

---

## 6. Example: Vector Searcher

Example use-case: cosine similarity, dot product, or L2 distance.

### Pattern

* Extract vector from Query.
* Iterate over all or some vectors.
* Compute similarity using algorithm of choice.
* Return scored docs.

Vector-based searchers often benefit from:

* SIMD operations
* ANN acceleration methods (HNSW, IVF, LSH)

---

## 7. Hybrid Searcher (Merging Multiple Searchers)

Hybrid search is where custom searchers shine. A hybrid searcher calls multiple searchers (e.g. BM25 + vector) and merges scores using custom weights.

This enables:

* Personalized ranking
* Domain-specific result shaping
* Stronger recall + precision

Hybrid searchers are also chainable—users can build their own meta-searchers.

---

## 8. Debugging Custom Searchers

Debug support:

* Searchers may print stats and weights
* Engine can enable `debug_mode` flag for verbose output
* Useful for scoring audits and tuning

Recommended debugging approach:

* Print token DF and internal math for text searchers
* Print vector norms and dot scores for vector searchers
* Log per-doc before/after merges for hybrid searchers

---

## 9. Best Practices

### ✔ Keep search pure

Do not mutate the index or shared state.

### ✔ Use cached global stats

Reuse IDF values or normalized vectors.

### ✔ Keep inner loops tight

Avoid allocations in hot loops.

### ✔ Respect extensibility

Do not hard-code assumptions about the index.

### ✔ Support future scoring updates

Design searchers so parameters (weights, boosts, cutoffs) can change.

---

## 10. Summary

Custom searchers are the primary extension point of Searus.
They allow users to:

* Add domain-specific algorithms
* Build hybrid ranking
* Experiment with research ideas
* Extend Searus beyond the core search types

Searchers define how results are scored, while the engine handles:

* Indexing
* Parallelization
* Merging
* Query parsing

This separation keeps extensions easy, clean, and powerful.
