use rustc_serialize::{json, Encodable, Decodable, Encoder};
use std::collections::HashMap;
use hyper;
use std::io::Read;

#[derive(Debug)]
pub struct Relationship<T = RelationshipUnidentifiedResult> {
    id: u64,
    type_name: String,
    from: u64,
    to: u64,
    data: Option<T>,
}

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

#[derive(RustcDecodable)]
struct RelationshipMetadataResult {
    id: u64,
}

#[derive(RustcDecodable)]
struct RelationshipResult<T: Decodable> {
    pub start: String,
    pub end: String,
    metadata: RelationshipMetadataResult,
    data: T,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct RelationshipUnidentifiedResult;

impl<T: Encodable + Decodable = RelationshipUnidentifiedResult> Relationship<T> {
    pub fn connect(cli: &::client::Client, id_from: u64, id_to: u64, type_name: String, properties: T) -> Result<Relationship<T>, String> {
        let mut rel_data:HashMap<String, RelationshipDataField<T>> = HashMap::new();
        let path: String = format!("/db/data/node/{}", id_to);
        rel_data.insert("to".to_string(), RelationshipDataField::Text(cli.build_uri(path)));
        rel_data.insert("type".to_string(), RelationshipDataField::Text(type_name.clone()));
        rel_data.insert("data".to_string(), RelationshipDataField::Data(properties));

        let rel_data_string = json::encode(&rel_data).unwrap();

        let path:String = format!("/db/data/node/{}/relationships", id_from);
        let mut res = cli.post(path)
            .body(&rel_data_string)
            .send()
            .unwrap();

        if hyper::status::StatusCode::Created != res.status {
            return Err("Network error".to_string());
        }

        let mut res_raw = String::new();
        let _ = res.read_to_string(&mut res_raw);
        let rel_json:RelationshipResult<T> = json::decode(&res_raw).unwrap();
        let rel = Relationship {
            id: rel_json.metadata.id,
            type_name: type_name.clone(),
            from: id_from,
            to: id_to,
            data: Some(rel_json.data),
        };

        info!("Relationship has been created");
        Ok(rel)
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
        assert!(node_parent.add(&cli));

        let mut node_child: node::Node = node::Node::new();
        assert!(node_child.add(&cli));

        let res: Result<relationship::Relationship, String> = relationship::Relationship::connect(&cli, node_parent.id.unwrap(), node_child.id.unwrap(), "Likes".to_string(), relationship::RelationshipUnidentifiedResult);
        assert!(res.is_ok());

        let rel = res.unwrap();
        assert!(rel.id > 0);
    }
}
