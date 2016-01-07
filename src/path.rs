use rustc_serialize::{json, Encodable, Decodable};
use std::collections::HashMap;
use std::rc::Rc;
use std::io::Read;
pub use types::Error;
use hyper;

#[derive(Debug)]
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

enum ResultNumericity {
    One,
    Multiple,
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

    pub fn path_with_depth(mut self, algorithm: Algorithm, max_depth: usize) -> PathBuilder {
        self.param.algorithm = (match algorithm {
            Algorithm::ShortestPath => "shortestPath",
            Algorithm::AllSimplePaths => "allSimplePaths",
            Algorithm::AllPaths => "allPaths",
            _ => panic!("Algorithm is not compatible with depth-only method: {:?}", algorithm),
        }).to_string();
        self.param.max_depth = Some(max_depth);
        self
    }

    pub fn path_with_weight(mut self, cost_property: String, default_cost: f64) -> PathBuilder {
        self.param.cost_property = Some(cost_property);
        self.param.default_cost = Some(default_cost);
        self.param.algorithm = "dijkstra".to_string();
        self
    }

    // pub fn relationships(mut self) -> PathBuilder {
    //     // TODO discover relationship settings form Neo4J api
    //     self
    // }

    pub fn get_all(&self) -> Result<Vec<Path>, Error> {
        self.get(ResultNumericity::Multiple)
    }

    pub fn get_one(&self) -> Result<Path, Error> {
        self.get(ResultNumericity::One)
    }

    fn get<T: Decodable>(&self, result_numericity: ResultNumericity) -> Result<T, Error> {
        let path = match result_numericity {
            ResultNumericity::One => format!("/db/data/node/{}/path", self.from),
            ResultNumericity::Multiple => format!("/db/data/node/{}/paths", self.from),
        };
        let payload = match json::encode(&self.param) {
            Ok(s) => s,
            _ => return Err(Error::DataError),
        };

        let mut res_war = String::new();
        let mut res = try_rest!(self.cli.as_ref().post(path).body(&*payload));
        let _ = res.read_to_string(&mut res_war);
        Ok(match json::decode::<T>(&res_war) {
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

    #[derive(RustcEncodable, RustcDecodable)]
    struct TestWeightedType {
        weight: f64,
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

    // 1, 2 and 3 are connected, 4 is single:
    // 1 -> 2 -> 3 | 4
    fn setup() -> (Rc<client::Client>, Vec<relationship::Relationship<TestWeightedType>>, Vec<node::Node>) {
        let cli = Rc::new(get_client());

        let mut node_1: node::Node = node::Node::new();
        assert!(node_1.add(cli.as_ref()).is_ok());
        let mut node_2: node::Node = node::Node::new();
        assert!(node_2.add(cli.as_ref()).is_ok());
        let mut node_3: node::Node = node::Node::new();
        assert!(node_3.add(cli.as_ref()).is_ok());
        let mut node_4: node::Node = node::Node::new();
        assert!(node_4.add(cli.as_ref()).is_ok());

        let rel_1: relationship::Relationship<TestWeightedType> = relationship::Relationship::connect(cli.as_ref(), node_1.get_id().unwrap(), node_2.get_id().unwrap(), "Relate".to_string(), Some(TestWeightedType { weight: 1.6 })).unwrap();
        let rel_2: relationship::Relationship<TestWeightedType> = relationship::Relationship::connect(cli.as_ref(), node_2.get_id().unwrap(), node_3.get_id().unwrap(), "Relate".to_string(), Some(TestWeightedType { weight: 2.1 })).unwrap();

        (cli, vec![rel_1, rel_2], vec![node_1, node_2, node_3, node_4])
    }

    #[test]
    fn test_get_no_shortest_path() {
        let (cli, rels, nodes) = setup();

        let path_builder = path::PathBuilder::new(cli.clone(), nodes[0].get_id().unwrap(), nodes[3].get_id().unwrap())
            .path_with_depth(path::Algorithm::ShortestPath, 3);
        let paths = path_builder.get_all().unwrap();
        assert_eq!(0, paths.len());

        for rel in rels {
            assert!(rel.delete(cli.as_ref()).is_ok());
        }
        for n in nodes {
            assert!(n.delete(cli.as_ref()).is_ok());
        }
    }

    #[test]
    fn test_get_one_shortest_path() {
        let (cli, rels, nodes) = setup();

        let path_builder = path::PathBuilder::new(cli.clone(), nodes[0].get_id().unwrap(), nodes[2].get_id().unwrap())
            .path_with_depth(path::Algorithm::ShortestPath, 3);
        let p = path_builder.get_one();
        assert!(p.is_ok());

        for rel in rels {
            assert!(rel.delete(cli.as_ref()).is_ok());
        }
        for n in nodes {
            assert!(n.delete(cli.as_ref()).is_ok());
        }
    }

    #[test]
    fn test_get_shortest_paths() {
        let (cli, rels, nodes) = setup();

        let path_builder = path::PathBuilder::new(cli.clone(), nodes[0].get_id().unwrap(), nodes[2].get_id().unwrap())
            .path_with_depth(path::Algorithm::ShortestPath, 3);
        let paths = path_builder.get_all().unwrap();
        assert_eq!(1, paths.len());

        for rel in rels {
            assert!(rel.delete(cli.as_ref()).is_ok());
        }
        for n in nodes {
            assert!(n.delete(cli.as_ref()).is_ok());
        }
    }

    #[test]
    fn test_get_one_all_simple_path() {
        let (cli, rels, nodes) = setup();

        let path_builder = path::PathBuilder::new(cli.clone(), nodes[0].get_id().unwrap(), nodes[2].get_id().unwrap())
            .path_with_depth(path::Algorithm::AllSimplePaths, 3);
        let p = path_builder.get_one();
        assert!(p.is_ok());

        for rel in rels {
            assert!(rel.delete(cli.as_ref()).is_ok());
        }
        for n in nodes {
            assert!(n.delete(cli.as_ref()).is_ok());
        }
    }

    #[test]
    fn test_get_one_all_path() {
        let (cli, rels, nodes) = setup();

        let path_builder = path::PathBuilder::new(cli.clone(), nodes[0].get_id().unwrap(), nodes[2].get_id().unwrap())
            .path_with_depth(path::Algorithm::AllPaths, 3);
        let p = path_builder.get_one();
        assert!(p.is_ok());

        for rel in rels {
            assert!(rel.delete(cli.as_ref()).is_ok());
        }
        for n in nodes {
            assert!(n.delete(cli.as_ref()).is_ok());
        }
    }

    #[test]
    fn test_get_weighted_path() {
        let (cli, rels, nodes) = setup();

        let path_builder = path::PathBuilder::new(cli.clone(), nodes[0].get_id().unwrap(), nodes[2].get_id().unwrap())
            .path_with_weight("weight".to_string(), 1.0);
        let p = path_builder.get_one().unwrap();
        assert_eq!(3.7, p.weight.unwrap());
        assert_eq!(2, p.directions.len());

        let rel_shortcut: relationship::Relationship<TestWeightedType> = relationship::Relationship::connect(cli.as_ref(), nodes[0].get_id().unwrap(), nodes[2].get_id().unwrap(), "Relate".to_string(), Some(TestWeightedType { weight: 0.5 })).unwrap();

        let path_builder = path::PathBuilder::new(cli.clone(), nodes[0].get_id().unwrap(), nodes[2].get_id().unwrap())
            .path_with_weight("weight".to_string(), 1.0);
        let p = path_builder.get_one().unwrap();
        assert_eq!(0.5, p.weight.unwrap());
        assert_eq!(1, p.directions.len());

        for rel in rels {
            assert!(rel.delete(cli.as_ref()).is_ok());
        }
        assert!(rel_shortcut.delete(cli.as_ref()).is_ok());
        for n in nodes {
            assert!(n.delete(cli.as_ref()).is_ok());
        }
    }
}