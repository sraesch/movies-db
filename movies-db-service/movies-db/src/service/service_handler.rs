use crate::{Error, Movie, MovieId, MovieSearchQuery, MovieStorage, MoviesIndex, Options};

use actix_web::{web, Responder, Result};
use log::error;
use serde::{Deserialize, Serialize};

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
