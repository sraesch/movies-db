use crate::{Error, MovieId, MovieStorage, MoviesIndex, Options};

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

    /// Handles the request to show the list of all movies.
    pub async fn handle_show_list(&self) -> Result<impl Responder> {
        let movies_ids = match self.index.search_movies(Default::default()) {
            Ok(movies_ids) => movies_ids,
            Err(err) => {
                error!("Internal error: {}", err);
                return Err(actix_web::error::ErrorInternalServerError(err));
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
}
