use std::sync::Arc;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use log::{error, info};

use crate::{Error, MovieStorage, MoviesIndex, Options};

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

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

        match self.run_http_server().await {
            Err(err) => {
                error!("Running the service...FAILED");
                error!("Error: {}", err);
                return Err(err);
            }
            Ok(()) => {
                info!("Running the service...STOPPED");
            }
        }

        Ok(())
    }

    async fn run_http_server(&self) -> Result<(), Error> {
        info!("Running the HTTP server...");

        info!("Listening on {}", self.options.http_address);

        match HttpServer::new(|| {
            App::new().service(web::resource("/").to(|| async { "hello world" }))
        })
        .bind(self.options.http_address.clone())?
        .run()
        .await
        {
            Err(err) => {
                error!("Running the HTTP server...FAILED");
                error!("Error: {}", err);
                Err(err.into())
            }
            Ok(_) => {
                info!("Running the HTTP server...STOPPED");
                Ok(())
            }
        }
    }

    // async fn serve_req(
    //     req: Request<Body>,
    //     shared: Arc<ServiceHandler<I, S>>,
    // ) -> Result<Response<Body>, BoxError> {
    //     match shared.handle(req).await {
    //         Ok(response) => Ok(response),
    //         Err(err) => panic!("Unexpected server error due to {}", err),
    //     }
    // }
}
