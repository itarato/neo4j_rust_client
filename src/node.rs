use std::io::Read;
use rustc_serialize::{json, Encodable, Decodable};
use hyper;

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

    pub fn get(client: &::client::Client, id: u64) -> Option<Node<T>> {
        let path: String = format!("/db/data/node/{}", id);
        let mut res_raw = String::new();
        let mut res = client.get(path)
            .send()
            .unwrap();

        if hyper::status::StatusCode::Ok != res.status {
            return None;
        }

        let mut node = Self::new();
        let _ = res.read_to_string(&mut res_raw);
        let node_json: NodeDataResponse<T> = json::decode(&res_raw).unwrap();
        node.update_from_response_node_json(node_json);

        Some(node)
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

    pub fn add(&mut self, client: &::client::Client) -> bool {
        let mut response_raw = String::new();
        let props_string: String = match self.properties {
            Some(ref props) => json::encode(props).unwrap(),
            None => String::new(),
        };

        let mut res = client.post("/db/data/node".to_string())
            .body(&props_string)
            .send()
            .unwrap();

        let _ = res.read_to_string(&mut response_raw);
        let node_json:NodeDataResponse<T> = json::decode(&response_raw).unwrap();
        self.update_from_response_node_json(node_json);

        info!("Node created, id: {}.", self.id.unwrap());
        hyper::status::StatusCode::Created == res.status
    }

    pub fn add_labels(&mut self, client: &::client::Client, labels: Vec<String>) -> bool {
        // TODO error if id does not exist
        let labels_raw:String = ["[\"", &*labels.join("\", \""), "\"]"].concat();
        let path:String = format!("/db/data/node/{}/labels", self.id.unwrap());
        let res = client.post(path)
            .body(&*labels_raw)
            .send()
            .unwrap();

        info!("Labels {:?} added to {}.", labels_raw, self.id.unwrap());
        hyper::status::StatusCode::NoContent == res.status
    }

    pub fn delete(self, client: &::client::Client) -> bool {
        let path:String = format!("/db/data/node/{}", self.id.unwrap());
        let res = client.delete(path)
            .send()
            .unwrap();
        hyper::status::StatusCode::NoContent == res.status
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
        assert!(node.add(&cli));
        assert!(node.id.is_some());

        let node_reload: Option<node::Node> = node::Node::get(&cli, node.id.unwrap());
        assert_eq!(node.id, node_reload.unwrap().id);

        assert!(node.delete(&cli));
    }

    #[test]
    pub fn test_node_create_with_type() {
        let node_data = TestNodeData {
            name: "foobar".to_string(),
            level: -42,
        };
        let cli = get_client();
        let mut node: node::Node<TestNodeData> = node::Node::new();
        node.set_properties(node_data);
        assert!(node.add(&cli));
        assert!(node.id.is_some());

        let node_reload: Option<node::Node<TestNodeData>> = node::Node::get(&cli, node.id.unwrap());
        assert_eq!(node.id, node_reload.unwrap().id);

        assert!(node.delete(&cli));
    }

    #[test]
    pub fn test_node_labels() {
        let cli = get_client();
        let mut node: node::Node = node::Node::new();
        assert!(node.add(&cli));
        assert!(node.id.is_some());

        assert!(node.add_labels(&cli, vec!["foo".to_string(), "bar".to_string()]));
        assert!(node.delete(&cli));
    }

}