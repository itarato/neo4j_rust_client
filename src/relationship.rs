use rustc_serialize::{json, Encodable, Encoder};
use std::collections::HashMap;
use hyper;

pub struct Relationship;
 // {
    // id: Option<u64>,
    // type_name: String,
    // from: u64,
    // to: u64,
    // data: Option<T>,
// }

enum RelationshipDataField<T: Encodable> {
    Text(String),
    Data(T),
}

impl<T: Encodable> Encodable for RelationshipDataField<T> {
    fn encode<S: Encoder>(&self, encoder: &mut S) -> Result<(), S::Error> {
        match *self {
            RelationshipDataField::Text(ref s) => encoder.emit_str(&*s),
            RelationshipDataField::Data(ref d) => d.encode(encoder),
        }
    }
}

// type RelationshipCollectionResult = Vec<Relationship>;

// struct RelationshipCollection;

// impl RelationshipCollection {
//     fn all_for_node(cli: &::client::Client, id: u64) -> RelationshipCollectionResult {
//         Vec::new()
//     }
// }

impl Relationship {
    pub fn connect<T: Encodable>(cli: &::client::Client, id_from: u64, id_to: u64, type_name: String, properties: T) -> bool {
        let mut rel_data:HashMap<String, RelationshipDataField<T>> = HashMap::new();
        let path: String = format!("/db/data/node/{}", id_to);
        rel_data.insert("to".to_string(), RelationshipDataField::Text(cli.build_uri(path)));
        rel_data.insert("type".to_string(), RelationshipDataField::Text(type_name));
        rel_data.insert("data".to_string(), RelationshipDataField::Data(properties));

        let rel_data_string = json::encode(&rel_data).unwrap();

        let path:String = format!("/db/data/node/{}/relationships", id_from);
        let res = cli.post(path)
            .body(&rel_data_string)
            .send()
            .unwrap();

        info!("Relationship has been created");
        hyper::status::StatusCode::Created == res.status
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use client;
    use relationship;
    use node;

    fn get_client() -> ::client::Client {
        let password = env::var("RUST_NEO4J_CLIENT_TEST_PASSWORD");
        let username = env::var("RUST_NEO4J_CLIENT_TEST_USERNAME");
        assert!(password.is_ok());
        assert!(username.is_ok());

        client::ClientBuilder::new()
            .credential(username.unwrap(), password.unwrap())
            .get()
    }

    #[test]
    pub fn test_connect_nodes() {
        let cli = get_client();

        let mut node_parent: node::Node = node::Node::new();
        node_parent.add(&cli);

        let mut node_child: node::Node = node::Node::new();
        node_child.add(&cli);

        assert!(relationship::Relationship::connect(&cli, node_parent.id.unwrap(), node_child.id.unwrap(), "Likes".to_string(), ()));
    }
}
