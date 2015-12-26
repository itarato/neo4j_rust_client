use rustc_serialize::{json, Encodable, Decodable, Encoder};
use std::collections::HashMap;
use hyper;
use std::io::Read;

#[derive(RustcDecodable, Debug)]
struct RelationshipMetadataResult {
    id: u64,
}

#[derive(RustcDecodable, Debug)]
struct RelationshipResult<T: Decodable> {
    pub start: String,
    pub end: String,
    metadata: RelationshipMetadataResult,
    data: T,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct RelationshipUnidentifiedResult;

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

#[derive(Debug)]
pub struct Relationship<T = RelationshipUnidentifiedResult> {
    id: u64,
    type_name: String,
    from: u64,
    to: u64,
    properties: Option<T>,
}

impl<T: Encodable + Decodable = RelationshipUnidentifiedResult> Relationship<T> {
    pub fn connect(cli: &::client::Client, id_from: u64, id_to: u64, type_name: String, properties: Option<T>) -> Result<Relationship<T>, String> {
        let mut rel_data:HashMap<String, RelationshipDataField<T>> = HashMap::new();
        let path: String = format!("/db/data/node/{}", id_to);
        rel_data.insert("to".to_string(), RelationshipDataField::Text(cli.build_uri(path)));
        rel_data.insert("type".to_string(), RelationshipDataField::Text(type_name.clone()));

        if properties.is_some() {
            rel_data.insert("data".to_string(), RelationshipDataField::Data(properties.unwrap()));
        }

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
            properties: Some(rel_json.data),
        };

        info!("Relationship has been created");
        Ok(rel)
    }

    pub fn delete(self, cli: &::client::Client) -> bool {
        let path:String = format!("/db/data/relationship/{}", self.id);
        let res = cli.delete(path)
            .send()
            .unwrap();

        info!("Relationship deleted: {}", self.id);
        hyper::status::StatusCode::NoContent == res.status
    }
}

pub type RelationshipCollectionResult = Vec<RelationshipResult<RelationshipUnidentifiedResult>>;

pub struct RelationshipCollection;

impl RelationshipCollection {
    pub fn all_for_node(cli: &::client::Client, id: u64) -> Result<Vec<Relationship>, String> {
        let path = format!("/db/data/node/{}/relationships/all", id);
        let mut res = cli.get(path)
            .send()
            .unwrap();

        if hyper::status::StatusCode::Ok != res.status {
            return Err("Request error".to_string());
        }

        let mut res_raw = String::new();
        let _ = res.read_to_string(&mut res_raw);
        let rels_result_object = json::Json::from_str(&res_raw).unwrap();
        let rels_result = rels_result_object.as_array().unwrap();

        let rels: Vec<Relationship> = rels_result.iter().map(|elem| {
            let obj = elem.as_object().unwrap();
            Relationship {
                id: obj.get("metadata").unwrap().as_object().unwrap().get("id").unwrap().as_u64().unwrap(),
                type_name: obj.get("type").unwrap().as_string().unwrap().to_string(),
                from: get_node_id_from_url(obj.get("start").unwrap().as_string().unwrap().to_string()),
                to: get_node_id_from_url(obj.get("end").unwrap().as_string().unwrap().to_string()),
                properties: None,
            }
        }).collect();
        Ok(rels)
    }
}

/******************************************************************************
 * Helper functions.
 */

 fn get_node_id_from_url(url: String) -> u64 {
    url.split("node/").collect::<Vec<&str>>().last().unwrap().parse::<u64>().unwrap()
}

/******************************************************************************
 * Tests.
 */

#[cfg(test)]
mod tests {
    use std::env;
    use client;
    use relationship;
    use node;

    #[derive(RustcEncodable, RustcDecodable)]
    struct TestRelationshipData {
        name: String,
        level: i64,
    }

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
    pub fn test_connect_nodes_no_type() {
        let cli = get_client();

        let mut node_parent: node::Node = node::Node::new();
        assert!(node_parent.add(&cli).is_ok());

        let mut node_child: node::Node = node::Node::new();
        assert!(node_child.add(&cli).is_ok());

        let res: Result<relationship::Relationship, String> = relationship::Relationship::connect(&cli, node_parent.get_id().unwrap(), node_child.get_id().unwrap(), "Likes".to_string(), None);
        assert!(res.is_ok());

        let rel = res.unwrap();
        assert!(rel.id > 0);

        assert!(rel.delete(&cli));
    }

    #[test]
    pub fn test_connect_nodes_with_type() {
        let cli = get_client();

        let mut node_parent: node::Node = node::Node::new();
        assert!(node_parent.add(&cli).is_ok());

        let mut node_child: node::Node = node::Node::new();
        assert!(node_child.add(&cli).is_ok());

        let res: Result<relationship::Relationship<TestRelationshipData>, String> = relationship::Relationship::connect(&cli, node_parent.get_id().unwrap(), node_child.get_id().unwrap(), "Likes".to_string(), Some(TestRelationshipData { name: "Steve".to_string(), level: -6, }));
        assert!(res.is_ok());

        let rel = res.unwrap();
        assert!(rel.id > 0);
        assert_eq!(rel.properties.as_ref().unwrap().name, "Steve");
        assert_eq!(rel.properties.as_ref().unwrap().level, -6);

        assert!(rel.delete(&cli));
    }
}
