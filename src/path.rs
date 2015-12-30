use rustc_serialize::{json, Encodable};
use std::collections::HashMap;
use std::rc::Rc;
use std::io::Read;
pub use types::Error;

pub enum Algorithm {
    ShortestPath,
    AllSimplePaths,
    AllPaths,
    Dijkstra,
}

pub enum RelationshipType {
    From,
    To,
}

pub enum RelationshipDirection {
    In,
    Out,
}

#[derive(RustcDecodable, Debug)]
pub struct Path {
    directions: Vec<String>,
    weight: Option<f64>,
    start: String,
    end: String,
    nodes: Vec<String>,
    length: usize,
    relationships: Vec<String>,
}

#[derive(RustcEncodable)]
struct PathBuilderParam {
    to: String,
    cost_property: Option<String>,
    default_cost: Option<f64>,
    max_depth: Option<usize>,
    relationships: Option<HashMap<String, String>>,
    algorithm: String,
}

impl PathBuilderParam {
    fn new() -> PathBuilderParam {
        PathBuilderParam {
            to: String::new(),
            cost_property: None,
            default_cost: None,
            max_depth: None,
            relationships: None,
            algorithm: String::new(),
        }
    }
}

pub struct PathBuilder {
    from: u64,
    param: PathBuilderParam,
    cli: Rc<::client::Client>,
}

impl PathBuilder {
    pub fn new(cli: Rc<::client::Client>, from: u64, to: u64) -> PathBuilder {
        let mut instance = PathBuilder {
            param: PathBuilderParam::new(),
            cli: cli,
            from: from,
        };

        instance.param.to = instance.cli.as_ref().build_uri(format!("/db/data/node/{}", to));

        instance
    }

    pub fn shortest_path(mut self, max_depth: usize) -> PathBuilder {
        self.param.algorithm = "shortestPath".to_string();
        self.param.max_depth = Some(max_depth);
        self
    }

    // pub fn relationships(mut self) -> PathBuilder {
    //     // TODO discover relationships settings form Neo4J api
    //     self
    // }

    pub fn get_all(&self) -> Result<Vec<Path>, Error> {
        let path = format!("/db/data/node/{}/paths", self.from);
        let payload = match json::encode(&self.param) {
            Ok(s) => s,
            _ => return Err(Error::DataError),
        };

        let mut res_war = String::new();
        let mut res = match self.cli.as_ref().post(path).body(&*payload).send() {
            Ok(res) => res,
            _ => return Err(Error::NetworkError),
        };
        let _ = res.read_to_string(&mut res_war);
        Ok(match json::decode(&res_war) {
            Ok(obj) => obj,
            _ => return Err(Error::DataError),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use client;
    use node;
    use path;
    use relationship;
    use std::rc::Rc;
    pub use types::Error;

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
    fn test_get_shortest_path() {
        let cli = Rc::new(get_client());

        let mut node_1: node::Node = node::Node::new();
        assert!(node_1.add(cli.as_ref()).is_ok());
        let mut node_2: node::Node = node::Node::new();
        assert!(node_2.add(cli.as_ref()).is_ok());
        let mut node_3: node::Node = node::Node::new();
        assert!(node_3.add(cli.as_ref()).is_ok());

        let path_builder = path::PathBuilder::new(cli.clone(), node_1.get_id().unwrap(), node_3.get_id().unwrap())
            .shortest_path(100);
        let paths = path_builder.get_all().unwrap();
        assert_eq!(0, paths.len());

        let rel_1: relationship::Relationship = relationship::Relationship::connect(cli.as_ref(), node_1.get_id().unwrap(), node_2.get_id().unwrap(), "Relate".to_string(), None).unwrap();
        let rel_2: relationship::Relationship = relationship::Relationship::connect(cli.as_ref(), node_2.get_id().unwrap(), node_3.get_id().unwrap(), "Relate".to_string(), None).unwrap();

        let paths_reloaded = path_builder.get_all().unwrap();
        assert_eq!(1, paths_reloaded.len());

        assert!(rel_1.delete(cli.as_ref()).is_ok());
        assert!(rel_2.delete(cli.as_ref()).is_ok());

        assert!(node_3.delete(cli.as_ref()).is_ok());
        assert!(node_2.delete(cli.as_ref()).is_ok());
        assert!(node_1.delete(cli.as_ref()).is_ok());
    }
}