# Extensions in Searus

Extensions provide a powerful mechanism to augment, intercept, and modify nearly every part of the Searus search pipeline. They are designed to give developers full control over query preprocessing, result postprocessing, caching, dynamic item fetching, and even altering search behavior before individual searchers execute.

Extensions allow Searus to behave like a **search middleware pipeline**.

---

# What Are Extensions?

Extensions are modular components that hook into the lifecycle of a Searus search request. They can:

* Transform or rewrite queries.
* Modify the list of items before searchers run.
* Add, remove, or replace searchers.
* Provide caching layers (per-query, per-searcher, global cache, etc.).
* Limit, filter, or postprocess results.
* Add or modify metadata on queries or results.
* Fetch external resources (e.g., database rows, web APIs) before search.

You can think of extensions as similar to:

* **Express.js middleware**
* **Kafka Streams processors**
* **Elasticsearch plugins**
* **Compiler passes**

Each extension can choose which hooks it implements.

---

# Extension Lifecycle Hooks

Extensions have a series of optional hooks representing each phase of a search:

1. **before_query(query)**

   * Called before validating or processing the query.
   * Modify text, tags, filters, or append additional metadata.

2. **before_items(query, items)**

   * Allows adding new items, removing items, or replacing items entirely.
   * Good for lazy loading, pagination, remote fetch, or pre-filtering.

3. **before_searcher(query, searcher, items)**

   * Lets you cancel searchers, change their config, or replace them.

4. **after_searcher(query, searcher, results)**

   * Adjust raw results from a single searcher.

5. **before_merge(query, all_searcher_results)**

   * Useful for weighting searchers, boosting certain types, etc.

6. **after_merge(query, merged_results)**

   * Modify the merged final result list before limiting.

7. **before_limit(query, merged_results)**

   * Best place for sorting and last-second prioritization.

8. **after_limit(query, final_results)**

   * Modify what is returned (e.g., add metadata, remove details, compress, etc.).

Extensions can implement any subset of these.

---

# Why Extensions Are Needed

Without extensions, Searus searchers only handle scoring. Extensions allow:

### ✔ Local or external caching

```
Query → Extension checks cache → return cached results (skip real search)
```

### ✔ Query expanding / rewriting

```
"ai" → ["ai", "machine learning", "deep learning"]
```

### ✔ Data fetching

```
Fetch extra items from API or DB before running search.
```

### ✔ Permissions and filtering

```
Filter out items user is not allowed to see.
```

### ✔ Rate limiting / throttling

### ✔ Custom ranking framework

---

# Extension Definition

Extensions should follow a unified trait:

```rust
pub trait SearusExtension {
    fn before_query(&self, query: &mut Query) {}
    fn before_items<T>(&self, query: &Query, items: &mut Vec<T>) {}
    fn before_searcher<T>(&self, query: &Query, searcher: &mut Box<dyn Searcher<T>>) {}
    fn after_searcher<T>(&self, query: &Query, results: &mut Vec<SearusMatch<T>>) {}
    fn before_merge<T>(&self, query: &Query, results: &mut Vec<SearusMatch<T>>) {}
    fn after_merge<T>(&self, query: &Query, results: &mut Vec<SearusMatch<T>>) {}
    fn before_limit<T>(&self, query: &Query, results: &mut Vec<SearusMatch<T>>) {}
    fn after_limit<T>(&self, query: &Query, results: &mut Vec<SearusMatch<T>>) {}
}
```

Each hook is optional.

---

# Example Extensions

## 1. Caching Extension

Caches the output of full search based on (query_hash, searcher_type):

```rust
struct CacheExt {
    cache: DashMap<u64, Vec<SearusMatch<Value>>> // or any T
}

impl SearusExtension for CacheExt {
    fn before_merge<T>(&self, query: &Query, _r: &mut Vec<SearusMatch<T>>) {}

    fn before_query(&self, query: &mut Query) {}

    fn after_limit<T>(&self, query: &Query, results: &mut Vec<SearusMatch<T>>) {
        let hash = query.hash();
        self.cache.insert(hash, results.clone());
    }
}
```

This cache extension can even *short-circuit*: if cache exists -> skip all searchers.

## 2. Query Rewriter Extension

```rust
struct QueryRewriteExt;

impl SearusExtension for QueryRewriteExt {
    fn before_query(&self, query: &mut Query) {
        if let Some(t) = &query.text {
            if t == "ml" {
                query.text = Some("machine learning".to_string());
            }
        }
    }
}
```

## 3. Dynamic Fetch Extension

Before search, pull items from a database:

```rust
struct FetchExt;

impl SearusExtension for FetchExt {
    fn before_items<T>(&self, _query: &Query, items: &mut Vec<T>) {
        let extra = fetch_from_somewhere();
        items.extend(extra);
    }
}
```

---

# How Extensions Fit Into SearusEngine

Engine pipeline becomes:

```
Query → extensions.before_query()
      → extensions.before_items()
      → For each searcher:
            before_searcher()
            run searcher
            after_searcher()
      → before_merge()
      → merge
      → after_merge()
      → before_limit()
      → apply skip/limit
      → after_limit()
      → return results
```

Extensions execute in order they were added:

```rust
let engine = SearusEngine::new()
    .with_extension(CacheExt::new())
    .with_extension(QueryRewriteExt);
```

---

# Extension Configuration

Extensions may have configuration:

* enable/disable per query
* priority ordering
* conflict-resolution rules
* shared state (Arc/Mutex or DashMap)

You may also want `ExtensionSet`, same as plugin systems.

---

# Best Extension Use Cases

* Distributed caching (Redis)
* Logging / analytics / telemetry
* Query monitoring
* Debugging search layouts
* Feature flags
* Personalized ranking (per user)
* Data enrichment

---

# Conclusion

Extensions are the power core of Searus. They allow the engine to:

* Become programmable
* Adapt to any environment
* Integrate external services
* Modify everything from query → results

They are essential for building a **production-tier, flexible search engine**.
