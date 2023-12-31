use std::{marker::PhantomData, sync::Arc};

use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{http::header, web, App, HttpServer, Responder, Result};

use log::{debug, error, info, trace};
use serde_qs::actix::QsQuery;
use tokio::sync::{mpsc, RwLock};

use crate::{
    ffmpeg::FFMpeg, service::preview_generator::PreviewGenerator, Error, Movie, MovieId,
    MovieSearchQuery, MovieStorage, MoviesIndex, Options,
};

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

use super::{preview_generator::ScreenshotRequest, service_handler::ServiceHandler};

use serde::{Deserialize, Serialize};

pub struct Service<I: Sized + 'static, S: Sized + 'static>
where
    I: MoviesIndex,
    S: MovieStorage,
{
    options: Options,
    phantom: PhantomData<(I, S)>,
}

/// The query for the GET /api/v1/movie endpoint.
#[derive(Debug, Deserialize, Serialize)]
struct MovieIdQuery {
    id: MovieId,
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
        let index = Arc::new(RwLock::new(I::new(&self.options)?));
        let storage = Arc::new(RwLock::new(S::new(&self.options)?));

        // create preview generator
        let ffmpeg = FFMpeg::new(&self.options.ffmpeg).await?;
        let preview_generator = PreviewGenerator::new(ffmpeg, index.clone(), storage.clone());
        let preview_requests = preview_generator.get_preview_request_sender();

        // spawn preview generator
        tokio::spawn(async move {
            let mut p = preview_generator;
            p.run().await;
        });

        // create handler
        let handler = self
            .create_service_handler(index.clone(), storage.clone(), preview_requests)
            .await?;
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
                .route("/movie", web::post().to(Self::handle_post_movie))
                .route("/movie", web::get().to(Self::handle_get_movie))
                .route("/movie", web::delete().to(Self::handle_delete_movie))
                .route("/movie/search", web::get().to(Self::handle_search_movie))
                .route("/movie/tags", web::get().to(Self::handle_get_tags))
                .route("/movie/file", web::post().to(Self::handle_upload_movie))
                .route("/movie/file", web::get().to(Self::handle_download_movie))
                .route(
                    "/movie/screenshot",
                    web::post().to(Self::handle_upload_screenshot),
                )
                .route(
                    "/movie/screenshot",
                    web::get().to(Self::handle_download_screenshot),
                );

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
                return Err(err.into());
            }
            Ok(_) => {
                info!("Running the HTTP server...STOPPED");
            }
        }

        Ok(())
    }

    /// Creates a new instance of the service handler.
    ///
    /// # Arguments
    /// * `index` - The movies index.
    /// * `storage` - The movie storage.
    /// * `preview_requests` - The channel to send preview requests to.
    async fn create_service_handler(
        &self,
        index: Arc<RwLock<I>>,
        storage: Arc<RwLock<S>>,
        preview_requests: mpsc::UnboundedSender<ScreenshotRequest>,
    ) -> Result<ServiceHandler<I, S>, Error> {
        info!("Creating the service handler...");
        match ServiceHandler::new(index, storage, preview_requests).await {
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

    /// Handles the POST /api/v1/movie endpoint.
    ///
    /// # Arguments
    /// * `handler` - The service handler.
    /// * `movie` - The movie to add.
    async fn handle_post_movie(
        handler: web::Data<RwLock<ServiceHandler<I, S>>>,
        movie: web::Json<Movie>,
    ) -> Result<impl Responder> {
        debug!("Handling POST /api/v1/movie");
        trace!("Request body: {:?}", movie);

        let movie: Movie = movie.into_inner();

        let handler = handler.read().await;
        handler.handle_add_movie(movie).await
    }

    /// Handles the GET /api/v1/movie endpoint.
    ///
    /// # Arguments
    /// * `handler` - The service handler.
    /// * `query` - The query parameters.
    async fn handle_search_movie(
        handler: web::Data<RwLock<ServiceHandler<I, S>>>,
        query: QsQuery<MovieSearchQuery>,
    ) -> Result<impl Responder> {
        debug!("Handling GET /api/v1/movie/search");
        trace!("Request query: {:?}", query);

        let query: MovieSearchQuery = query.into_inner();

        let handler = handler.read().await;

        handler.handle_search_movies(query).await
    }

    /// Handles the GET /api/v1/tags endpoint.
    ///
    /// # Arguments
    /// * `handler` - The service handler.
    async fn handle_get_tags(
        handler: web::Data<RwLock<ServiceHandler<I, S>>>,
    ) -> Result<impl Responder> {
        debug!("Handling GET /api/v1/movie/tags");

        let handler = handler.read().await;

        handler.handle_get_tags().await
    }

    /// Handles the GET /api/v1/movie endpoint.
    ///
    /// # Arguments
    /// * `handler` - The service handler.
    /// * `query` - The query parameters.
    async fn handle_get_movie(
        handler: web::Data<RwLock<ServiceHandler<I, S>>>,
        query: web::Query<MovieIdQuery>,
    ) -> Result<impl Responder> {
        debug!("Handling GET /api/v1/movie");
        trace!("Request query: {:?}", query);

        let id: MovieId = query.into_inner().id;

        let handler = handler.read().await;

        handler.handle_get_movie(id).await
    }

    /// Handles the DELETE /api/v1/movie endpoint.
    ///
    /// # Arguments
    /// * `handler` - The service handler.
    /// * `query` - The query parameters.
    async fn handle_delete_movie(
        handler: web::Data<RwLock<ServiceHandler<I, S>>>,
        query: web::Query<MovieIdQuery>,
    ) -> Result<impl Responder> {
        debug!("Handling DELETE /api/v1/movie");
        trace!("Request query: {:?}", query);

        let id: MovieId = query.into_inner().id;

        let handler = handler.read().await;

        handler.handle_delete_movie(id).await
    }

    /// Handles the POST /api/v1/movie/file endpoint.
    ///
    /// # Arguments
    /// * `handler` - The service handler.
    /// * `query` - The query parameters.
    /// * `multipart` - The multipart data.
    async fn handle_upload_movie(
        handler: web::Data<RwLock<ServiceHandler<I, S>>>,
        query: web::Query<MovieIdQuery>,
        multipart: Multipart,
    ) -> Result<impl Responder> {
        debug!("Handling POST /api/v1/movie/file");
        trace!("Request query: {:?}", query);

        let id: MovieId = query.into_inner().id;

        let handler = handler.read().await;

        handler.handle_upload_movie(id, multipart).await
    }

    /// Handles the GET /api/v1/movie/file endpoint.
    ///
    /// # Arguments
    /// * `handler` - The service handler.
    /// * `query` - The query parameters.
    async fn handle_download_movie(
        handler: web::Data<RwLock<ServiceHandler<I, S>>>,
        ranges: web::Header<header::Range>,
        query: web::Query<MovieIdQuery>,
    ) -> Result<impl Responder> {
        debug!("Handling GET /api/v1/movie/file");
        trace!("Request query: {:?}", query);

        let ranges: header::Range = ranges.0;

        let ranges = match ranges {
            header::Range::Bytes(ranges) => {
                trace!("Request ranges: {:?}", ranges);
                ranges
            }
            _ => {
                error!("Invalid range header");
                return Err(actix_web::error::ErrorRangeNotSatisfiable(
                    "Invalid range header",
                ));
            }
        };

        let id: MovieId = query.into_inner().id;

        let handler = handler.read().await;

        handler.handle_download_movie(id, &ranges).await
    }

    /// Handles the POST /api/v1/movie/screenshot endpoint.
    ///
    /// # Arguments
    /// * `handler` - The service handler.
    /// * `query` - The query parameters.
    /// * `multipart` - The multipart data.
    async fn handle_upload_screenshot(
        handler: web::Data<RwLock<ServiceHandler<I, S>>>,
        query: web::Query<MovieIdQuery>,
        multipart: Multipart,
    ) -> Result<impl Responder> {
        debug!("Handling POST /api/v1/movie/screenshot");
        trace!("Request query: {:?}", query);

        let id: MovieId = query.into_inner().id;

        let handler = handler.read().await;

        handler.handle_upload_screenshot(id, multipart).await
    }

    /// Handles the GET /api/v1/movie/screenshot endpoint.
    ///
    /// # Arguments
    /// * `handler` - The service handler.
    /// * `query` - The query parameters.
    async fn handle_download_screenshot(
        handler: web::Data<RwLock<ServiceHandler<I, S>>>,
        query: web::Query<MovieIdQuery>,
    ) -> Result<impl Responder> {
        debug!("Handling GET /api/v1/movie/screenshot");
        trace!("Request query: {:?}", query);

        let id: MovieId = query.into_inner().id;

        let handler = handler.read().await;

        handler.handle_download_screenshot(id).await
    }
}
