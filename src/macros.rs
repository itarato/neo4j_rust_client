#![macro_use]

macro_rules! request_fn {
    ($name:ident, $req_type:ident) => {
        pub fn $name(&self, path: String) -> hyper::client::RequestBuilder {
            self.request(hyper::method::Method::$req_type, path)
        }
    }
}

macro_rules! expect_code {
    ($response:expr, $code:ident) => (
        if hyper::status::StatusCode::$code != $response.status {
            return Err(Error::ResponseError);
        }
    )
}

macro_rules! try_rest {
    ($query:expr) => (
        try_rest!($query, Ok);
    );
    ($query:expr, $code:ident) => (
        {
            let response = match $query.send() {
                Ok(response) => response,
                Err(_) => return Err(Error::NetworkError),
            };
            expect_code!(response, $code);
            response
        }
    );
}
