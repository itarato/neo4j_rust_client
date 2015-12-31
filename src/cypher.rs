use rustc_serialize::{json, Encodable};
pub use types::Error;
use hyper;

#[derive(RustcEncodable)]
struct CypherStatement<T> {
    statement: String,
    parameters: Option<T>,
}

#[derive(RustcEncodable)]
struct CypherStatements<T: Encodable> {
    statements: Vec<CypherStatement<T>>,
}

pub struct Cypher;

impl Cypher {
    pub fn query<T: Encodable>(cli: &::client::Client, statement: String, parameters: T) -> Result<(), Error> {
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
        let res = match cli.post("/db/data/transaction/commit".to_string()).body(&payload).send() {
            Ok(res) => res,
            _ => return Err(Error::NetworkError),
        };
        if hyper::status::StatusCode::Ok != res.status {
            return Err(Error::ResponseError);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use client;
    use cypher;

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
        let res = cypher::Cypher::query(&cli, "START n=node(1) RETURN n".to_string(), ());
        assert!(res.is_ok());
    }
}