use actix_cors::Cors;
use actix_web::{web, App, HttpServer, Responder, Result};

use log::{debug, error, info, trace};
use tokio::sync::RwLock;

use crate::{Error, Options};

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

use super::service_handler::ServiceHandler;

use serde::{Deserialize, Serialize};

pub struct Service
{
    options: Options,
}

/// The query for the GET /api/v1/movie endpoint.
#[derive(Debug, Deserialize, Serialize)]
struct MovieIdQuery {
    id: String,
}

impl Service
{
    /// Creates a new instance of the service.
    ///
    /// # Arguments
    /// * `options` - The options for the service.
    pub fn new(options: &Options) -> Result<Self, Error> {
        let options = options.clone();

        Ok(Self { options })
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

    /// Runs the HTTP server.
    async fn run_http_server(&self) -> Result<(), Error> {
        let handler = self.create_service_handler()?;
        let handler = RwLock::new(handler);
        let handler = web::Data::new(handler);

        info!("Running the HTTP server...");
        info!("Listening on {}", self.options.http_address);

        match HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_header()
                .allow_any_method()
                .allow_any_origin();

            let api_v1 = web::scope("/api/v1")
                .route("/preview", web::post().to(Self::handle_post_preview));

            App::new()
                .wrap(cors)
                .app_data(handler.clone())
                .service(api_v1)
        })
        .bind(self.options.http_address)?
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

    /// Creates a new instance of the service handler.
    fn create_service_handler(&self) -> Result<ServiceHandler, Error> {
        info!("Creating the service handler...");
        match ServiceHandler::new(self.options.clone()) {
            Err(err) => {
                error!("Creating the service handler...FAILED");
                error!("Error: {}", err);
                Err(err)
            }
            Ok(handler) => {
                info!("Creating the service handler...OK");
                Ok(handler)
            }
        }
    }

    /// Handles the POST /api/v1/preview endpoint.
    ///
    /// # Arguments
    /// * `handler` - The service handler.
    /// * `query` - The query parameters.
    async fn handle_post_preview(
        handler: web::Data<RwLock<ServiceHandler>>,
        query: web::Query<MovieIdQuery>,
    ) -> Result<impl Responder> {
        debug!("Handling GET /api/v1/movie");
        trace!("Request query: {:?}", query);

        let id: String = query.into_inner().id;

        let handler = handler.read().await;

        handler.handle_create_preview(&id).await
    }
}
