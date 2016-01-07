use rustc_serialize::{json};
use std::collections::HashMap;
use hyper;
pub use types::Error;

#[cfg(test)] extern crate rand;

pub struct Index {
    property_key: String,
    label: String,
}

impl Index {
    pub fn new(label: String, property_key: String) -> Index {
        Index {
            property_key: property_key,
            label: label,
        }
    }

    pub fn create(&self, cli: &::client::Client) -> Result<(), Error> {
        let path = format!("/db/data/schema/index/{}", self.label);

        let mut payload_data: HashMap<String, Vec<String>> = HashMap::new();
        payload_data.insert("property_keys".to_string(), vec![self.property_key.clone()]);
        let payload = json::encode(&payload_data).unwrap();

        try_rest!(cli.post(path).body(&*payload), Ok);
        Ok(())
    }

    pub fn delete(&self, cli: &::client::Client) -> Result<(), Error> {
        let path = format!("/db/data/schema/index/{}/{}", self.label, self.property_key);
        try_rest!(cli.delete(path), NoContent);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use client;
    use rand::{thread_rng, Rng};
    use index;

    fn get_client() -> ::client::Client {
        let password = env::var("RUST_NEO4J_CLIENT_TEST_PASSWORD");
        let username = env::var("RUST_NEO4J_CLIENT_TEST_USERNAME");
        assert!(password.is_ok());
        assert!(username.is_ok());

        client::ClientBuilder::new()
            .credential(username.unwrap(), password.unwrap())
            .get()
    }

    fn get_random_string(len: usize) -> String {
        thread_rng().gen_ascii_chars().take(len).collect()
    }

    #[test]
    fn test_index_add_and_remove() {
        let cli = get_client();
        let idx_name = get_random_string(16);
        let prop_name = get_random_string(16);
        let idx = index::Index::new(idx_name, prop_name);
        let res_create = idx.create(&cli);
        assert!(res_create.is_ok());

        let res_del = idx.delete(&cli);
        assert!(res_del.is_ok());
    }
}