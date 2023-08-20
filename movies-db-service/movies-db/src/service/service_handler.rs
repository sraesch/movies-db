use crate::{
    Error, Movie, MovieDataType, MovieId, MovieSearchQuery, MovieStorage, MoviesIndex, Options,
    ReadResource,
};

use actix_multipart::Multipart;
use actix_web::body::SizedStream;
use actix_web::web::Bytes;
use actix_web::HttpResponse;
use actix_web::{web, Responder, Result};
use futures::{Stream, StreamExt, TryStreamExt};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio_util::codec::{BytesCodec, FramedRead};

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
    pub async fn handle_add_movie(&mut self, movie: Movie) -> Result<impl Responder> {
        match self.index.add_movie(movie) {
            Ok(movie_id) => match self.storage.allocate_movie_data(movie_id.clone()).await {
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
    pub async fn handle_get_movie(&self, id: MovieId) -> Result<impl Responder> {
        match self.index.get_movie(&id) {
            Ok(movie) => Ok(web::Json(movie)),
            Err(err) => Self::handle_error(err),
        }
    }

    /// Handles the request to delete a new movie.
    ///
    /// # Arguments
    /// * `movie` - The movie to get.
    pub async fn handle_delete_movie(&mut self, id: MovieId) -> Result<impl Responder> {
        match self.index.remove_movie(&id) {
            Ok(()) => match self.storage.remove_movie_data(id).await {
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
                .await
            {
                Ok(writer) => writer,
                Err(err) => {
                    return Self::handle_error(err);
                }
            };

            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.next().await {
                let data = match chunk {
                    Ok(data) => data,
                    Err(err) => {
                        error!("Error reading chunk: {}", err);
                        return Err(actix_web::error::ErrorInternalServerError(err));
                    }
                };

                match writer.write_all(&data).await {
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

    /// Handles the request to upload a movie.
    ///
    /// # Arguments
    /// * `id` - The id of the movie to upload.
    pub async fn handle_download_movie(&self, id: MovieId) -> Result<impl Responder> {
        info!("Downloading movie {} ...", id);

        // get the movie file info, needed for requesting the movie data
        let movie_file_info = match self.index.get_movie(&id) {
            Ok(movie) => match movie.movie_file_info {
                Some(movie_file_info) => movie_file_info,
                None => {
                    error!("Movie {} has no movie file info", id);
                    return Err(actix_web::error::ErrorConflict(format!(
                        "Movie {} is not yet ready",
                        id
                    )));
                }
            },
            Err(err) => {
                error!("Error getting movie info: {}", err);
                return Self::handle_error(err);
            }
        };

        // create reader onto the movie data
        let movie_data = match self
            .storage
            .read_movie_data(
                id,
                MovieDataType::MovieData {
                    ext: movie_file_info.extension.clone(),
                },
            )
            .await
        {
            Ok(movie_data) => movie_data,
            Err(err) => {
                error!("Error reading movie data: {}", err);
                return Self::handle_error(err);
            }
        };

        // create response
        let mut res = HttpResponse::Ok();
        res.content_type(movie_file_info.mime_type);

        FramedRead::new(movie_data, BytesCodec::new());

        todo!("Implement streaming of movie data");

        // let stream =
        // SizedStream::new(movie_data.get_size() as u64, stream);

        // Ok(HttpResponse::Ok().content_type(movie_file_info.mime_type))
        //     .body(movie_data))
        Ok(res)
    }

    /// Handles the request to show the list of all movies.
    pub async fn handle_search_movies(&self, query: MovieSearchQuery) -> Result<impl Responder> {
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

// struct StreamReadResource<R: ReadResource> {
//     inner: R,
//     buf: [u8; 1024],
// }

// impl<R: ReadResource> Unpin for StreamReadResource<R> {}

// impl<R: ReadResource> StreamReadResource<R> {
//     pub fn new(inner: R) -> Self {
//         Self {
//             inner,
//             buf: [0u8; 1024],
//         }
//     }
// }

// impl<R: ReadResource> Stream for StreamReadResource<R> {
//     type Item = std::io::Result<Bytes>;

//     fn poll_next(
//         mut self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Option<Self::Item>> {
//         let s = self.deref_mut();

//         let reader = &mut s.inner;
//         let buffer = &mut s.buf;

//         match reader.read(buffer.as_mut()) {
//             Ok(0) => std::task::Poll::Ready(None),
//             Ok(n) => std::task::Poll::Ready(Some(Ok(Bytes::copy_from_slice(&buffer[..n])))),
//             Err(err) => std::task::Poll::Ready(Some(Err(err))),
//         }
//     }
// }
