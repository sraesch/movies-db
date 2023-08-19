use std::{marker::PhantomData, sync::RwLock};

use actix_web::{get, post, web, App, HttpServer, Responder, Result};

use log::{error, info};

use crate::{Error, MovieStorage, MoviesIndex, Options};

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

use super::service_handler::ServiceHandler;

pub struct Service<I: Sized + 'static, S: Sized + 'static>
where
    I: MoviesIndex,
    S: MovieStorage,
{
    options: Options,
    phantom: PhantomData<(I, S)>,
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
        let options = options.clone();
        let phantom = PhantomData {};

        Ok(Self { phantom, options })
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
            let api_v1 =
                web::scope("/api/v1").route("/movie", web::get().to(Self::handle_show_list));

            App::new().app_data(handler.clone()).service(api_v1)
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

    /// Creates a new instance of the service handler.
    fn create_service_handler(&self) -> Result<ServiceHandler<I, S>, Error> {
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

    async fn handle_show_list(
        handler: web::Data<RwLock<ServiceHandler<I, S>>>,
    ) -> Result<impl Responder> {
        handler.read().unwrap().handle_show_list().await
    }
}
