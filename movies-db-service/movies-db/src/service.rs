use std::convert::Infallible;

use hyper::service::{make_service_fn, service_fn};
use log::info;

use crate::{Error, MoviesIndex, Options};

pub struct Service<I>
where
    I: MoviesIndex,
{
    index: I,
}

impl<I> Service<I>
where
    I: MoviesIndex,
{
    /// Creates a new instance of the service.
    ///
    /// # Arguments
    /// * `options` - The options for the service.
    pub fn new(options: &Options) -> Result<Self, Error> {
        let index = I::new(options)?;

        Ok(Self { index })
    }

    /// Runs the service.
    pub async fn run(&self) -> Result<(), Error> {
        info!("Running the service...");

        Ok(())
    }

    fn run_http_server(&self) {
        info!("Running the HTTP server...");

        // create service handler
        let make_svc = make_service_fn(move |_conn| {
            async move {
                // This is the request handler.
                Ok::<_, Infallible>(service_fn(move |req| Self::serve_req(req, handler)))
            }
        });
    }
}
