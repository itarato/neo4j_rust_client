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

    // Add labels.
    node.add_labels(&cli, vec!["foo".to_string(), "bar".to_string()]);

    // Delete node.
    node.delete(&cli);
}
```

# Client

Client is necessary for the resources to operate on. Setting up a credential is optional.

```rust
let cli = client::ClientBuilder::new()
    .credential("myusername".to_string(), "mypassword".to_string())
    .get();
```

Check if the connection is correct and established:

```rust
if !cli.is_alive() {
    // Error handling.
}
```

# Node

Creating an empty (type-less) node:

```rust
let n: node::Node = node::Node::new().add(&cli).unwrap();

// Verify:
println!("New node ID is: {}", n.get_id().unwrap());
```

Creating node with data (typed):

```rust
#[derive(RustcDecodable)]
struct MyData {
    name: String,
    level: f64,
}

let mut n: node::Node<MyData> = node::Node::new();
n.set_properties(MyData {
   name: "Acme Corp".to_string(),
   level: 10.2,
});
n.add(&cli);
```

Add labels:

```rust
let mut n = /* fetch or create */;
n.add_labels(&cli);
```

Fetch node (type-less and typed):

```rust
// Without data:
let n = node::Node::get(&cli, <NODE_ID>).unwrap();

// With data:
let n: node::Node<MyData> = node::Node::get(&cli, <NODE_ID>).unwrap();

// Assuming T from Node<T> implements Display.
println!("Node with id: {} has labels: {:?} and properties: {:?}", node.get_id().unwrap(), node.get_labels(), node.get_properties().unwrap());
```

Delete node:

```rust
n.delete(&cli);
```

# Index

Add index for a property:

```rust
index::Index::new("name_index".to_string(), "name".to_string()).create(&cli);
```

Delete existing index:

```rust
index::Index::new("name_index".to_string(), "name".to_string()).delete(&cli);
```

# Relationships




Test (for developers)
---------------------

Get the latest Neo4j runtime and start the server.

```shell
export RUST_NEO4J_CLIENT_TEST_PASSWORD=<...>; export RUST_NEO4J_CLIENT_TEST_USERNAME=<...>; cargo test
```
