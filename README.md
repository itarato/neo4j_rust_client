Neo4j Rust Client
=================

This is an unofficial (play project) Rust client for Neo4J graph database.

*Do not use it on production! This is only an educational project.*


Usage
-----

Example scenario for creating connection, adding a node and setting some attributes on it.

```rust
extern crate neo4j_client;

use neo4j_client::{client, node};
use rustc_serialize::{Encodable};

#[derive(RustcEncodable)]
struct MyNodeType {
    name: String,
}

fn main() {
    // Create a client connection.
    let cli = client::ClientBuilder::new()
        .credential("myusername".to_string(), "mypassword".to_string())
        .get();

    // Make up the data for the graph node.
    let mut node: node::Node<MyNodeType> = node::Node::new();
    node.set_properties(MyNodeType {
        name: "John Doe",
    });

    // Create node.
    node.add(&cli);
    assert!(node.id.is_some());
    assert!(node.properties.name, "John Doe");

    // Add labels.
    node.add_labels(&cli, vec!["foo".to_string(), "bar".to_string()]);

    // Delete node.
    node.delete(&cli);
}
```


Test (for developers)
---------------------

Get the latest Neo4j runtime and start the server.

```shell
export RUST_NEO4J_CLIENT_TEST_PASSWORD=<...>; export RUST_NEO4J_CLIENT_TEST_USERNAME=<...>; cargo test
```
