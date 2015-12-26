#[derive(Debug)]
pub enum Error {
    NetworkError(String),
    DataError(String),
}
