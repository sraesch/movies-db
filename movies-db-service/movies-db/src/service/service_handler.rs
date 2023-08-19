use log::debug;

use hyper::{
    body::to_bytes,
    header::{
        ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
        ACCESS_CONTROL_MAX_AGE, CONTENT_TYPE,
    },
    Body, HeaderMap, Method, Request, Response, StatusCode,
};

use std::convert::Infallible;

use crate::{MovieStorage, MoviesIndex, Options};

pub struct ServiceHandler<I, S>
where
    I: MoviesIndex,
    S: MovieStorage,
{
    index: I,
    storage: S,
}

impl<I, S> ServiceHandler<I, S>
where
    I: MoviesIndex,
    S: MovieStorage,
{
    /// Creates a new instance of the service handler.
    ///
    /// # Arguments
    /// * `options` - The options for the service handler.
    pub fn new(options: Options) -> Self {
        let index = I::new(&options).unwrap();
        let storage = S::new(&options).unwrap();

        Self { index, storage }
    }

    /// Handles the HTTP request.
    ///
    /// # Arguments
    /// * `req` - The HTTP request to handle.
    pub async fn handle(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        debug!("Got Request {:?}", req);

        // check for options request
        if req.method() == Method::OPTIONS {
            let mut response = Self::status_ok();
            Self::patch_cors_header(response.headers_mut());

            return Ok(response);
        }

        let uri = req.uri().clone();
        let path = uri.path();

        if path.starts_with("/api/v1/") {
            return self.handle_v1(req).await;
        } else {
            Ok(Self::invalid_request(
                "Invalid or missing version".to_owned(),
            ))
        }
    }

    /// Handles the HTTP request for the v1 API.
    ///
    /// # Arguments
    /// * `req` - The HTTP request to handle.
    async fn handle_v1(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let uri = req.uri().clone();
        let path = uri.path();

        if path == "/api/v1/movies" {
            match req.method() {
                Method::GET => {
                    let response = self.handle_v1_movies_get(req).await;
                    return Ok(response);
                }
                _ => {
                    let mut response = Self::invalid_request("Invalid Request".to_owned());
                    Self::patch_cors_header(response.headers_mut());

                    return Ok(response);
                }
            }
        }

        let mut response = Self::status_ok();
        Self::patch_cors_header(response.headers_mut());

        Ok(response)
    }

    /// Handles the HTTP GET request for the v1 API by returning a list of movies.
    ///
    /// # Arguments
    /// * `req` - The HTTP request to handle.
    async fn handle_v1_movies_get(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let mut response = Self::status_ok();
        Self::patch_cors_header(response.headers_mut());

        let movies = self.index.list_movies();
        let movies = serde_json::to_string(&movies).unwrap();

        *response.body_mut() = Body::from(movies);

        Ok(response)
    }

    /// Sets the CORS headers in the header map.
    ///
    /// # Arguments
    /// * `header` - The header map to set the CORS headers in.
    fn patch_cors_header(header: &mut HeaderMap) {
        header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
        header.insert(ACCESS_CONTROL_ALLOW_METHODS, "*".parse().unwrap());
        header.insert(ACCESS_CONTROL_ALLOW_HEADERS, "*".parse().unwrap());
        header.insert(ACCESS_CONTROL_MAX_AGE, "1728000".parse().unwrap());
    }

    /// Creates a new response with status code 200.
    fn status_ok() -> Response<Body> {
        let response = Response::new(Body::empty());
        response
    }

    /// Creates a new response with status code 400.
    ///
    /// # Arguments
    /// * `err_str` - The error string to return in the response.
    fn invalid_request(err_str: String) -> Response<Body> {
        let mut response = Response::new(Body::from(err_str));

        response
            .headers_mut()
            .insert(CONTENT_TYPE, "text/plain".parse().unwrap());

        *response.status_mut() = StatusCode::BAD_REQUEST;

        response
    }
}
