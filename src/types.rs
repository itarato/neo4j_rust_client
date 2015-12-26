#[derive(Debug)]
pub enum Error {
    NetworkError,
    ResponseError,
    DataError,
    IntegrityError,
}
