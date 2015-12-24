Neo4j Rust Client
=================

This is an unofficial (play project) Rust client for Neo4J graph database.

*Do not use it on production! This is only an educational project.*


Usage
-----

```rust
extern crate neo4j_client;

use neo4j_client::{client, node};
use rustc_serialize::{Encodable};

#[derive(RustcEncodable)]
struct MyNodeType {
    name: String,
}

fn main() {
    let cli = client::ClientBuilder::new()
        .credential("myusername".to_string(), "mypassword".to_string())
        .get();

    let mut node: node::Node<MyNodeType> = node::Node::new();
    node.set_properties(MyNodeType {
        name: "John Doe",
    });
    node::add(&cli);
    assert!(node.id.is_some());
}
```


Test (for developers)
---------------------

Get the latest Neo4j runtime and start the server.

```shell
export RUST_NEO4J_CLIENT_TEST_PASSWORD=<...>; export RUST_NEO4J_CLIENT_TEST_USERNAME=<...>; cargo test
```
