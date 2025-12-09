use searus::prelude::*;

#[derive(Clone)]
struct User {
    name: String,
    id: i32,
    username: String,
}

#[derive(Clone)]
struct Post {
    title: String,
    content: String,
    user: User,
}

fn main() {
    let posts: Vec<Post> = get_some_posts();

    let engine = SearusEngine::new()
        .with(SemanticSearch::new());

    let query = SemanticQuery::new("some text");

    let rules = SemanticRules::new()
        .field("title", SemanticField::new()
            .matcher(Matcher::Fuzzy)
            .priority(1))
        .field("content", SemanticField::new()
            .matcher(Matcher::Tokenized)
            .priority(2))
        .object("user", SemanticFieldObject::direct()
            .field("username", SemanticField::new()
                .matcher(Matcher::Exact)
                .priority(3))
            .no_match())
        .no_match();

    // returns Vec<SearusMatch<T>>
    let results = engine.search(posts, rules, query, 
        SearchOptions::new().skip(10).limit(100));

    for r in results {
        println!("score={} title={}", r.score, r.item.title);
    }
}
