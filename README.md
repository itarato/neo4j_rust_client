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
let n: node::Node<MyData> = node::Node::get(&cli, 123).unwrap();

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

Establish new relationship - no properties (type-less):

```rust
let res: Result<relationship::Relationship, Error> = relationship::Relationship::connect(&cli, 17, 42, "FriendsWith".to_string(), None);
```

With props (typed):

```rust
#[derive(RustcEncodable, RustcDecodable)]
struct TestRelationshipData {
    name: String,
    level: i64,
}

let res: Result<relationship::Relationship<TestRelationshipData>, Error> = relationship::Relationship::connect(&cli, 17, 42, "FriendsWith".to_string(), Some(TestRelationshipData { name: "strong".to_string(), level: 60, }));
```

Load connection (with known prop type) by its known ID:

```rust
#[derive(RustcEncodable, RustcDecodable)]
struct TestRelationshipData {
    name: String,
    level: i64,
}

let res: Result<relationship::Relationship<TestRelationshipData>, Error> = relationship::Relationship::get(&cli, 123);
```

Load all connections from/to a node (collection can be fetched without properties only):

```rust
let rels = relationship::RelationshipCollection::all_for_node(&cli, 42).unwrap();
```

Separately set one property on a connection:

```rust
let rel: relationship::Relationship<TestRelationshipData> = relationship::Relationship::get(&cli, 123).unwrap();
rel.set_property(&cli, "name".to_string(), "complicated".to_string());
```

Delete a connection:

```rust
rel.delete(&cli);
```

# Paths and graph algorithms

For details about the Neo4J part visit http://neo4j.com/docs/stable/rest-api-graph-algos.html.

Setting up a path request:

```rust
let max_depth = 3;
let path_builder = path::PathBuilder::new(Rc::new(cli), 17, 42)
    .path_with_depth(path::Algorithm::ShortestPath, max_depth);
```

Get one result (depending on the algorithm, the shortest or just one random):

```rust
path_builder.get_one().unwrap();
```

Get all:

```rust
path_builder.get_all().unwrap();
```

Use Dijkstra with weights:

```rust
let default_weight = 1.0;
let path = path::PathBuilder::new(Rc::new(cli), 17, 42)
    .path_with_weight("weight".to_string(), default_weight)
    .get_one()
    .unwrap();
```

# Cypher queries and transactions

Make a query and get the result:

```rust
#[derive(RustcDecodable)]
struct QueryResult {
    row: Vec<String>,
}

let mut params = HashMap::new();
params.insert("id".to_string(), node.get_id().unwrap());

let res = cypher::Cypher::query::<HashMap<String, u64>, Vec<QueryResult>>(&cli, "START n=node({id}) RETURN n.name".to_string(), params);

println!("First result is: {:?}", res.unwrap().results[0].data[0].row[0])
```

Make a transaction (in this example without query parameters or return type):

```rust
let mut trans = cypher::CypherTransaction::new(Rc::new(cli));
trans.query("CREATE (n) RETURN n".to_string(), ());
trans.commit();
// Or: trans.rollback();
```

Test (for developers)
---------------------

Get the latest Neo4j runtime and start the server.

```shell
export RUST_NEO4J_CLIENT_TEST_PASSWORD=<...>; export RUST_NEO4J_CLIENT_TEST_USERNAME=<...>; cargo test
```


# Run tests:

```bash
 RUST_NEO4J_CLIENT_TEST_USERNAME=<username> RUST_NEO4J_CLIENT_TEST_PASSWORD=<password> cargo test
```