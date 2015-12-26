use std::io::Read;
use rustc_serialize::{json, Encodable, Decodable};
use hyper;
pub use types::Error;

pub struct Node <T: Encodable = NodeUnidentifiedData> {
    id: Option<u64>,
    labels: Vec<String>,
    properties: Option<T>,
}

#[derive(RustcDecodable, RustcEncodable)]
struct NodeMetadataResponse {
    id: u64,
    labels: Vec<String>,
}

#[derive(RustcDecodable, RustcEncodable)]
struct NodeDataResponse<T: Decodable = NodeUnidentifiedData> {
    metadata: NodeMetadataResponse,
    data: T,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct NodeUnidentifiedData;

impl<T: Encodable + Decodable> Node<T> {
    pub fn new() -> Node<T> {
        Node {
            id: None,
            labels: Vec::new(),
            properties: None,
        }
    }

    pub fn get(client: &::client::Client, id: u64) -> Result<Node<T>, Error> {
        let path: String = format!("/db/data/node/{}", id);
        let mut res_raw = String::new();
        let mut res = match client.get(path).send() {
            Ok(res) => res,
            Err(_) => return Err(Error::NetworkError),
        };

        if hyper::status::StatusCode::Ok != res.status {
            return Err(Error::ResponseError);
        }

        let mut node = Self::new();
        let _ = res.read_to_string(&mut res_raw);
        let node_json: NodeDataResponse<T> = match json::decode(&res_raw) {
            Ok(res) => res,
            Err(_) => return Err(Error::DataError),
        };
        node.update_from_response_node_json(node_json);

        Ok(node)
    }

    pub fn get_id(&self) -> Option<u64> {
        self.id
    }

    fn update_from_response_node_json(&mut self, node_json: NodeDataResponse<T>) {
        // TODO check collision if exist and a different would be set
        self.id = Some(node_json.metadata.id);
        self.labels = node_json.metadata.labels.clone();
        self.properties = Some(node_json.data);
    }

    pub fn set_properties(&mut self, props: T) {
        self.properties = Some(props);
    }

    pub fn add(&mut self, client: &::client::Client) -> Result<(), Error> {
        if self.get_id().is_some() {
            return Err(Error::IntegrityError);
        }

        let mut response_raw = String::new();
        let props_string: String = match self.properties {
            Some(ref props) => json::encode(props).unwrap(),
            None => String::new(),
        };

        let mut res = match client.post("/db/data/node".to_string()).body(&props_string).send() {
            Ok(res) => res,
            _ => return Err(Error::NetworkError),
        };
        if hyper::status::StatusCode::Created != res.status {
            return Err(Error::ResponseError);
        }

        let _ = res.read_to_string(&mut response_raw);
        let node_json:NodeDataResponse<T> = match json::decode(&response_raw) {
            Ok(s) => s,
            _ => return Err(Error::DataError),
        };
        self.update_from_response_node_json(node_json);

        info!("Node created, id: {}", self.get_id().unwrap());
        Ok(())
    }

    pub fn add_labels(&mut self, client: &::client::Client, labels: Vec<String>) -> Result<(), Error> {
        if self.get_id().is_none() {
            return Err(Error::IntegrityError);
        }

        let labels_raw:String = ["[\"", &*labels.join("\", \""), "\"]"].concat();
        let path:String = format!("/db/data/node/{}/labels", self.id.unwrap());
        let res = match client.post(path).body(&*labels_raw).send() {
            Ok(res) => res,
            _ => return Err(Error::NetworkError),
        };

        if hyper::status::StatusCode::NoContent != res.status {
            return Err(Error::NetworkError);
        }

        info!("Labels {:?} added to {}", labels_raw, self.id.unwrap());
        Ok(())
    }

    pub fn delete(self, client: &::client::Client) -> Result<(), Error> {
        if self.get_id().is_none() {
            return Err(Error::IntegrityError);
        }

        let path:String = format!("/db/data/node/{}", self.get_id().unwrap());
        let res = match client.delete(path).send() {
            Ok(res) => res,
            _ => return Err(Error::NetworkError),
        };

        if hyper::status::StatusCode::NoContent != res.status {
            return Err(Error::ResponseError);
        }

        info!("Node deleted: {}", self.id.unwrap());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use client;
    use node;
    use rustc_serialize::{Encodable};

    #[derive(RustcEncodable, RustcDecodable)]
    struct TestNodeData {
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
    pub fn test_node_create_no_type() {
        let cli = get_client();
        let mut node: node::Node = node::Node::new();
        assert!(node.add(&cli).is_ok());
        assert!(node.get_id().is_some());

        let node_reload: node::Node = node::Node::get(&cli, node.get_id().unwrap()).unwrap();
        assert_eq!(node.get_id(), node_reload.get_id());
        assert_eq!(node_reload.labels.len(), 0);

        assert!(node.delete(&cli).is_ok());
    }

    #[test]
    pub fn test_node_create_with_type() {
        let node_data = TestNodeData {
            name: "John Doe".to_string(),
            level: -42,
        };
        let cli = get_client();
        let mut node: node::Node<TestNodeData> = node::Node::new();
        node.set_properties(node_data);
        assert!(node.add(&cli).is_ok());
        assert!(node.get_id().is_some());

        let node_reload: node::Node<TestNodeData> = node::Node::get(&cli, node.get_id().unwrap()).unwrap();
        assert_eq!(node.get_id(), node_reload.get_id());
        assert_eq!(node_reload.properties.as_ref().unwrap().name, "John Doe");
        assert_eq!(node_reload.properties.as_ref().unwrap().level, -42);
        assert_eq!(node_reload.labels.len(), 0);

        let delete_res = node.delete(&cli);
        assert!(delete_res.is_ok());
    }

    #[test]
    pub fn test_node_labels() {
        let cli = get_client();
        let mut node: node::Node = node::Node::new();
        assert!(node.add(&cli).is_ok());
        assert!(node.id.is_some());

        assert!(node.add_labels(&cli, vec!["foo".to_string(), "bar".to_string()]).is_ok());

        let node_reload: node::Node = node::Node::get(&cli, node.get_id().unwrap()).unwrap();
        assert_eq!(node_reload.labels.len(), 2);
        assert!(node_reload.labels.iter().any(|label| label == "foo"));
        assert!(node_reload.labels.iter().any(|label| label == "bar"));

        assert!(node.delete(&cli).is_ok());
    }

}