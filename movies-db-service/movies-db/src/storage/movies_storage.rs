use tokio::io::{AsyncRead, AsyncWrite};

use crate::{Error, MovieId, Options};

use async_trait::async_trait;

/// The type of data to store.
pub enum MovieDataType {
    MovieData {
        /// The file extension of the movie data.
        ext: String,
    },
    ScreenshotData {
        /// The file extension of the screenshot data.
        ext: String,
    },
}

/// The trait for reading movie data.
#[async_trait]
pub trait ReadResource: AsyncRead + Unpin + 'static {
    async fn get_size(&self) -> usize;
}

/// The trait for storing movie data.
#[async_trait]
pub trait MovieStorage: Send + Sync {
    type W: AsyncWrite + Unpin;
    type R: ReadResource;

    /// Creates a new instance of the storage.
    ///
    /// # Arguments
    /// * `options` - The options to use for the storage.
    fn new(options: &Options) -> Result<Self, Error>
    where
        Self: Sized;

    /// Allocates space for a new movie.
    ///
    /// # Arguments
    /// * `id` - The movie id for which to allocate the data.
    async fn allocate_movie_data(&self, id: MovieId) -> Result<(), Error>;

    /// Returns a writer for the given movie id and data type to store the data.
    ///
    /// # Arguments
    /// * `id` - The movie id for which to store the data.
    /// * `data_type` - The type of data to store.
    async fn write_movie_data(
        &self,
        id: MovieId,
        data_type: MovieDataType,
    ) -> Result<Self::W, Error>;

    /// Removes the data for the given movie id.
    ///
    /// # Arguments
    /// * `id` - The movie id for which to remove the data.
    async fn remove_movie_data(&self, id: MovieId) -> Result<(), Error>;

    /// Returns a reader for the given movie id and data type to read the data.
    ///
    /// # Arguments
    /// * `id` - The movie id for which to read the data.
    /// * `data_type` - The type of data to read.
    async fn read_movie_data(
        &self,
        id: MovieId,
        data_type: MovieDataType,
    ) -> Result<Self::R, Error>;
}
