# SEARUS — TAG RELATIONSHIP TREE (TRT).md

> Documentation for implementing and using the **Tag Relationship Tree (TRT)** feature in Searus' TaggedSearch module. TRT enables hierarchical or semantic expansion of tag-based searches, allowing related tags to contribute to search results with weighted strength.

---

## Table of contents

1. Overview
2. Purpose
3. Structure of a TRT
4. Relationship depth & propagation
5. Relationship properties & weights
6. Integration with TaggedSearch
7. Query usage
8. Example
9. Notes & best practices

---

## 1. Overview

A **Tag Relationship Tree (TRT)** is a user-provided data structure that defines relationships between tags. It allows the search engine to expand queries to include related tags while applying weaker scores based on their relationship strength and distance in the tree.

This enables searches like:

* Query: `ai`
* TRT: `ai` → `machine learning` → `python`
* Search result: Posts tagged `python` are included but with **reduced strength**.

---

## 2. Purpose

* Enhance **tagged search** by considering related tags.
* Enable **flexible weighting** of search results based on tag proximity.
* Support **complex semantic relationships**.
* Give developers control over **relation rules, depth, and strength decay**.

---

## 3. Structure of a TRT

A TRT is represented as a **vector of nodes**, where each node represents a tag and its relationships. Each relationship maps a related tag to a strength value (0 < strength <= 1).

Example structure in Rust:

```rust
struct TagNode {
    tag: String,
    relationships: HashMap<String, f32>, // related_tag -> strength
}

let trt_nodes = vec![
    TagNode { tag: "ai".to_string(), relationships: hashmap!{"machine learning".to_string() => 0.7} },
    TagNode { tag: "machine learning".to_string(), relationships: hashmap!{"ai".to_string() => 0.9, "python".to_string() => 0.4} },
    TagNode { tag: "python".to_string(), relationships: hashmap!{"programming".to_string() => 0.6} },
];
```

* Relationships are **bidirectional** if specified. The engine follows the relationships up to the depth limit.

---

## 4. Relationship depth & propagation

* Users provide a **maximum depth** for TRT expansion.
* Each level further from the original query tag contributes **weaker search strength**, multiplied by the edge strength.
* Example:

  * Query tag: `ai` (depth = 2)
  * `machine learning` (depth 1) → result strength multiplied by 0.7
  * `python` (depth 2) → result strength multiplied by 0.7 * 0.4 = 0.28

---

## 5. Relationship properties & weights

Each TRT node/edge defines:

* `tag`: string
* `relationships`: HashMap of related tags to strength values

Strength propagation:

* Result strength = `original_tag_score * product_of_strengths_along_path`
* Ensures farther relatives appear weaker in search results

---

## 6. Integration with TaggedSearch

* TRT is optional and provided when initializing `TaggedSearch`:

```rust
let trt = TagRelationshipTree::new(trt_nodes);
let tagged_search = TaggedSearch::new().with_trt(trt);
```

* During query, if `with_trt(depth)` is enabled, engine traverses TRT up to the supplied depth and adjusts candidate scores.

---

## 7. Query usage

* `query_tags`: Vec<String> — tags to search for
* `with_trt(depth: usize)` — enables TRT expansion up to given depth
* Result scoring:

  * Base score from direct tag matches
  * Weakened score for related tags, computed as product of relationship strengths along the path

Example:

```rust
let depth = 4;
let query = Query::builder()
    .tags(vec!["ai".to_string()])
    .with_trt(depth)
    .build();

let results = engine.search::<Post>(posts, query);
```

---

## 8. Example

Suppose the TRT nodes are:

```
ai (strength to machine learning=0.7)
machine learning (strength to ai=0.9, python=0.4)
python (strength to programming=0.6)
```

* Query: `ai` with depth=3
* Matching posts:

  1. Tagged `ai` → strength 1.0
  2. Tagged `machine learning` → strength 0.7
  3. Tagged `python` → strength 0.7 * 0.4 = 0.28
  4. Tagged `programming` → strength 0.7 * 0.4 * 0.6 = 0.168

The engine merges and sorts candidates by **adjusted strength**.

---

## 9. Notes & best practices

* Avoid cycles in TRT; engine should track visited tags to prevent infinite loops.
* Use meaningful `strength` values; too low may make related tags irrelevant.
* For large TRTs, consider caching flattened expansions per query.
* Users can mix **direct tag matches** and **TRT-expanded matches** for flexibility.
* TRTs are fully optional; without a TRT, `TaggedSearch` behaves like a standard tag filter.

---

**Summary:**

The Tag Relationship Tree (TRT) feature enables hierarchical or semantic tag expansion with weighted scoring. Users provide a vector of nodes, each with a `tag` and a mapping of `relationships` with strength values. During search, TRT expansion propagates scores along relationships up to a specified depth, allowing distant tags to contribute weaker but relevant search results.
