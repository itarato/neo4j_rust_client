use std::io::Read;
use rustc_serialize::{json, Encodable};

pub struct Node <T: Encodable> {
    id: Option<u64>,
    properties: Option<T>,
}

impl<T: Encodable> Node<T> {
    pub fn new() -> Node<T> {
        Node {
            id: None,
            properties: None,
        }
    }

    pub fn get(client: &::client::Client, id: u64) -> Option<Node<T>> {
        let path: String = format!("/db/data/node/{}", id);
        let mut res_raw = String::new();
        let _ = client.get(path)
            .send()
            .unwrap()
            .read_to_string(&mut res_raw);
        println!("{:?}", res_raw);

        None
    }

    pub fn set_properties(&mut self, props: T) {
        self.properties = Some(props);
    }

    pub fn add(&mut self, client: &::client::Client) {
        let mut response_raw = String::new();
        let props_string: String = match self.properties {
            Some(ref props) =>  json::encode(props).unwrap(),
            None => String::new(),
        };

        let _ = client.post("/db/data/node".to_string())
            .body(&props_string)
            .send()
            .unwrap()
            .read_to_string(&mut response_raw);

        let res = json::Json::from_str(&response_raw).unwrap();
        self.id = Some(res.as_object().unwrap().get("metadata").unwrap().as_object().unwrap().get("id").unwrap().as_u64().unwrap());

        info!("Node created, id: {}.", self.id.unwrap());
    }

    pub fn add_labels(&mut self, client: &::client::Client, labels: Vec<String>) {
        // TODO error if id does not exist
        let labels_raw:String = ["[\"", &*labels.join("\", \""), "\"]"].concat();
        let path:String = format!("/db/data/node/{}/labels", self.id.unwrap());
        client.post(path)
            .body(&*labels_raw)
            .send()
            .unwrap();

        info!("Labels {:?} added to {}.", labels_raw, self.id.unwrap());
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use client;
    use node;
    use rustc_serialize::{Encodable};

    #[derive(RustcEncodable)]
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
        let mut node: node::Node<()> = node::Node::new();
        node.add(&cli);
        assert!(node.id.is_some());
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
        node.add(&cli);
        assert!(node.id.is_some());
    }
}