extern crate hyper;

use rustc_serialize::base64;
use rustc_serialize::base64::ToBase64;
use hyper::header::{Accept, AcceptCharset, Authorization, Charset, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};

pub struct Client {
    url: String,
    port: u32,
    client: hyper::Client,
    headers: hyper::header::Headers,
}

pub struct ClientBuilder {
     client: Client,
}

impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        let mut headers = hyper::header::Headers::new();
        headers.set(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))]));
        headers.set(AcceptCharset(vec![qitem(Charset::Ext("utf-8".to_owned()))]));

        info!("Connection created");
        ClientBuilder {
            client: Client {
                url: "http://localhost".to_string(),
                port: 7474,
                client: hyper::Client::new(),
                headers: headers,
            },
        }
    }

    pub fn url(mut self, url: String) -> ClientBuilder {
        self.client.url = url.clone();
        self
    }

    pub fn port(mut self, port: u32) -> ClientBuilder {
        self.client.port = port;
        self
    }

    pub fn credential(mut self, username: String, password: String) -> ClientBuilder {
        let credential = format!("{}:{}", username, password).to_string().into_bytes()[..].to_base64(base64::STANDARD);
        let credential_token = format!("Basic <{}>", credential);
        self.client.headers.set(Authorization(credential_token));
        self
    }

    pub fn get(self) -> Client {
        self.client
    }
}

impl Client {
    pub fn build_uri(&self, path: String) -> String {
        format!("{}:{}{}", self.url, self.port, path)
    }

    pub fn is_alive(&self) -> bool {
        match self.get("/db/data".to_string()).send() {
            Ok(res) => hyper::status::StatusCode::Ok == res.status,
            _ => false,
        }
    }

    fn request(&self, method: hyper::method::Method, path: String) -> hyper::client::RequestBuilder {
        self.client.request(method, &self.build_uri(path)).headers(self.headers.clone())
    }

    request_fn! { get, Get }
    request_fn! { post, Post }
    request_fn! { delete, Delete }
    request_fn! { put, Put }
}

#[cfg(test)]
mod tests {
    use std::env;
    use client;

    #[test]
    pub fn test_connection_establish() {
        let password = env::var("RUST_NEO4J_CLIENT_TEST_PASSWORD");
        let username = env::var("RUST_NEO4J_CLIENT_TEST_USERNAME");
        assert!(password.is_ok());
        assert!(username.is_ok());

        let neo4j_client = client::ClientBuilder::new()
            .credential(username.unwrap(), password.unwrap())
            .get();

        assert!(neo4j_client.is_alive());
    }
}