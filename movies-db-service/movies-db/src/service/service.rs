use std::{convert::Infallible, sync::Arc};

use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};
use log::info;

use crate::{Error, MovieStorage, MoviesIndex, Options};

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

use hyper::{Body, Request, Response};

use super::service_handler::ServiceHandler;

pub struct Service<I, S>
where
    I: MoviesIndex,
    S: MovieStorage,
{
    handler: Arc<ServiceHandler<I, S>>,
    options: Options,
}

impl<I, S> Service<I, S>
where
    I: MoviesIndex,
    S: MovieStorage,
{
    /// Creates a new instance of the service.
    ///
    /// # Arguments
    /// * `options` - The options for the service.
    pub fn new(options: &Options) -> Result<Self, Error> {
        let handler: ServiceHandler<I, S> = ServiceHandler::new(options.clone());
        let handler = Arc::new(handler);

        let options = options.clone();

        Ok(Self { handler, options })
    }

    /// Runs the service.
    pub async fn run(&self) -> Result<(), Error> {
        info!("Running the service...");

        Ok(())
    }

    fn run_http_server(&self) {
        info!("Running the HTTP server...");

        let handler = self.handler.clone();

        // create service handler
        let make_svc = make_service_fn(move |_conn| {
            async move {
                // This is the request handler.
                Ok::<_, Infallible>(service_fn(move |req| Self::serve_req(req, handler)))
            }
        });

        // create server instance
        let server = Server::bind(&self.options.http_address).serve(make_svc);
    }

    async fn serve_req(
        req: Request<Body>,
        shared: Arc<ServiceHandler<I, S>>,
    ) -> Result<Response<Body>, BoxError> {
        match shared.handle(req).await {
            Ok(response) => Ok(response),
            Err(err) => panic!("Unexpected server error due to {}", err),
        }
    }
}
