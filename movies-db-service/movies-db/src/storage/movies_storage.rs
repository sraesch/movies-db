use std::io::{Read, Write};

use crate::{Error, MovieId, Options};

/// The type of data to store.
pub enum MovieDataType {
    MovieData {
        /// The file extension of the movie data.
        ext: String,
    },
}

/// The trait for storing movie data.
pub trait MovieStorage: Send + Sync {
    type W: Write;
    type R: Read;

    /// Creates a new instance of the storage.
    ///
    /// # Arguments
    /// * `options` - The options to use for the storage.
    fn new(options: &Options) -> Result<Self, Error>
    where
        Self: Sized;

    /// Returns a writer for the given movie id and data type to store the data.
    ///
    /// # Arguments
    /// * `id` - The movie id for which to store the data.
    /// * `data_type` - The type of data to store.
    fn write_movie_data(&self, id: MovieId, data_type: MovieDataType) -> Result<Self::W, Error>;

    /// Removes the data for the given movie id.
    ///
    /// # Arguments
    /// * `id` - The movie id for which to remove the data.
    fn remove_movie_data(&self, id: MovieId) -> Result<(), Error>;

    /// Returns a reader for the given movie id and data type to read the data.
    ///
    /// # Arguments
    /// * `id` - The movie id for which to read the data.
    /// * `data_type` - The type of data to read.
    fn read_movie_data(&self, id: MovieId, data_type: MovieDataType) -> Result<Self::R, Error>;
}
