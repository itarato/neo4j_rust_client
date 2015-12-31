use rustc_serialize::{json, Encodable, Decodable};
pub use types::Error;
use hyper;
use std::io::Read;

#[derive(RustcEncodable)]
struct CypherStatement<T> {
    statement: String,
    parameters: Option<T>,
}

#[derive(RustcEncodable)]
struct CypherStatements<T: Encodable> {
    statements: Vec<CypherStatement<T>>,
}

#[derive(RustcDecodable)]
pub struct CypherResult<T: Decodable> {
    pub columns: Vec<String>,
    pub data: T,
}

#[derive(RustcDecodable)]
pub struct CypherResultsResponse<T: Decodable> {
    pub results: Vec<CypherResult<T>>,
    pub errors: Vec<String>,
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct CypherUnidentifiedData;

pub struct Cypher;

impl Cypher {
    pub fn query<E: Encodable = (), D: Decodable = CypherUnidentifiedData>(cli: &::client::Client, statement: String, parameters: E) -> Result<CypherResultsResponse<D>, Error> {
        let statement = CypherStatement {
            statement: statement,
            parameters: Some(parameters),
        };
        let statements = CypherStatements {
            statements: vec![statement],
        };
        let payload = match json::encode(&statements) {
            Ok(s) => s,
            _ => return Err(Error::DataError),
        };
        let mut res = match cli.post("/db/data/transaction/commit".to_string()).body(&payload).send() {
            Ok(res) => res,
            _ => return Err(Error::NetworkError),
        };
        if hyper::status::StatusCode::Ok != res.status {
            return Err(Error::ResponseError);
        }

        let mut res_raw = String::new();
        let _ = res.read_to_string(&mut res_raw);

        let result: CypherResultsResponse<D> = match json::decode(&res_raw) {
            Ok(obj) => obj,
            _ => return Err(Error::DataError),
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use client;
    use cypher;
    use node;
    use rustc_serialize::{Encodable};
    use std::collections::HashMap;

    #[derive(RustcEncodable, RustcDecodable)]
    struct TestNodeProps {
        name: String,
    }

    #[derive(RustcDecodable)]
    struct TestQueryResult {
        row: Vec<String>,
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
    pub fn test_simple_query_with_commit() {
        let cli = get_client();

        let mut node = node::Node::new();
        node.set_properties(TestNodeProps { name: "Steve".to_string() });
        assert!(node.add(&cli).is_ok());

        let mut params = HashMap::new();
        params.insert("id".to_string(), node.get_id().unwrap());
        let res = cypher::Cypher::query::<HashMap<String, u64>, Vec<TestQueryResult>>(&cli, "START n=node({id}) RETURN n.name".to_string(), params);
        assert!(res.is_ok());

        assert_eq!(res.unwrap().results[0].data[0].row[0], "Steve");

        assert!(node.delete(&cli).is_ok());
    }
}