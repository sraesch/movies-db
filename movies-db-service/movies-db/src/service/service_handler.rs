use crate::{
    Error, Movie, MovieDataType, MovieId, MovieSearchQuery, MovieStorage, MoviesIndex, Options,
};

use actix_multipart::Multipart;
use actix_web::{web, Responder, Result};
use futures::{StreamExt, TryStreamExt};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{io::Write, path::PathBuf};

pub struct ServiceHandler<I, S>
where
    I: MoviesIndex,
    S: MovieStorage,
{
    index: I,
    storage: S,
}

#[derive(Debug, Serialize, Deserialize)]
struct MovieListEntry {
    id: MovieId,
    title: String,
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
    pub fn new(options: Options) -> Result<Self, Error> {
        let index = I::new(&options)?;
        let storage = S::new(&options)?;

        Ok(Self { index, storage })
    }

    /// Handles the request to add a new movie.
    ///
    /// # Arguments
    /// * `movie` - The movie to add.
    pub fn handle_add_movie(&mut self, movie: Movie) -> Result<impl Responder> {
        match self.index.add_movie(movie) {
            Ok(movie_id) => match self.storage.allocate_movie_data(movie_id.clone()) {
                Ok(()) => Ok(web::Json(movie_id)),
                Err(err) => Self::handle_error(err),
            },
            Err(err) => Self::handle_error(err),
        }
    }

    /// Handles the request to get a new movie.
    ///
    /// # Arguments
    /// * `movie` - The movie to get.
    pub fn handle_get_movie(&self, id: MovieId) -> Result<impl Responder> {
        match self.index.get_movie(&id) {
            Ok(movie) => Ok(web::Json(movie)),
            Err(err) => Self::handle_error(err),
        }
    }

    /// Handles the request to delete a new movie.
    ///
    /// # Arguments
    /// * `movie` - The movie to get.
    pub fn handle_delete_movie(&mut self, id: MovieId) -> Result<impl Responder> {
        match self.index.remove_movie(&id) {
            Ok(()) => match self.storage.remove_movie_data(id) {
                Ok(_) => Ok(actix_web::HttpResponse::Ok()),
                Err(err) => {
                    error!("Error deleting movie: {}", err);
                    Err(actix_web::error::ErrorInternalServerError(err))
                }
            },
            Err(err) => Self::handle_error(err),
        }
    }

    /// Handles the request to upload a movie.
    ///
    /// # Arguments
    /// * `id` - The id of the movie to upload.
    /// * `multipart` - The multipart data of the movie.
    pub async fn handle_upload_movie(
        &mut self,
        id: MovieId,
        mut multipart: Multipart,
    ) -> Result<impl Responder> {
        info!("Uploading movie {} ...", id);

        // iterate over multipart stream
        while let Ok(Some(mut field)) = multipart.try_next().await {
            // extract the filename
            let content_type = field.content_disposition();
            let filename: PathBuf = match content_type.get_filename() {
                Some(filename) => PathBuf::from(filename),
                None => {
                    error!("Invalid filename");
                    return Err(actix_web::error::ErrorBadRequest("Invalid filename"));
                }
            };

            debug!("Uploading file: {:?}", filename);

            // extract the extension
            let ext = match filename.extension() {
                Some(ext) => match ext.to_str() {
                    Some(ext) => ext.to_string(),
                    None => {
                        error!("Invalid extension");
                        return Err(actix_web::error::ErrorBadRequest("Invalid extension"));
                    }
                },
                None => {
                    error!("Invalid extension");
                    return Err(actix_web::error::ErrorBadRequest("Invalid extension"));
                }
            };

            debug!("Uploading file with extension: {:?}", ext);

            // open writer for storing movie data
            let mut writer = match self
                .storage
                .write_movie_data(id.clone(), MovieDataType::MovieData { ext })
            {
                Ok(writer) => writer,
                Err(err) => {
                    return Self::handle_error(err);
                }
            };

            // // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(data) => data,
                    Err(err) => {
                        error!("Error reading chunk: {}", err);
                        return Err(actix_web::error::ErrorInternalServerError(err));
                    }
                };

                match writer.write_all(&data) {
                    Ok(_) => (),
                    Err(err) => {
                        error!("Error writing chunk: {}", err);
                        return Err(actix_web::error::ErrorInternalServerError(err));
                    }
                }
            }
        }

        info!("Uploading movie {} ... DONE", id);

        Ok(actix_web::HttpResponse::Ok())
    }

    /// Handles the request to show the list of all movies.
    pub fn handle_search_movies(&self, query: MovieSearchQuery) -> Result<impl Responder> {
        let movies_ids = match self.index.search_movies(query) {
            Ok(movies_ids) => movies_ids,
            Err(err) => {
                error!("Error searching: {}", err);
                return Self::handle_error(err);
            }
        };

        let movies: Vec<MovieListEntry> = movies_ids
            .iter()
            .map(|id| {
                let movie = self.index.get_movie(id).unwrap();
                MovieListEntry {
                    id: id.clone(),
                    title: movie.movie.title,
                }
            })
            .collect();

        Ok(web::Json(movies))
    }

    fn handle_error<T: Responder>(err: Error) -> Result<T> {
        match err {
            Error::InvalidArgument(e) => {
                error!("Invalid argument: {}", e);
                Err(actix_web::error::ErrorBadRequest(e))
            }
            Error::NotFound(e) => {
                error!("Not found: {}", e);
                Err(actix_web::error::ErrorNotFound(e))
            }
            _ => {
                error!("Internal error: {}", err);
                Err(actix_web::error::ErrorInternalServerError(err))
            }
        }
    }
}
