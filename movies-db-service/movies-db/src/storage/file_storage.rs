use std::{
    fs::{create_dir_all, File},
    path::PathBuf,
};

use log::info;

use crate::{Error, MovieId, Options};

use super::movies_storage::{MovieDataType, MovieStorage};

pub struct FileStorage {
    root_dir: PathBuf,
}

impl MovieStorage for FileStorage {
    type W = File;
    type R = File;

    fn new(options: &Options) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let root_dir = options.root_dir.clone();

        // make sure the root directory exists
        create_dir_all(&root_dir).map_err(|e| {
            Error::Internal(format!(
                "Failed to create root directory '{}': {}",
                root_dir.display(),
                e
            ))
        })?;

        Ok(Self { root_dir })
    }

    fn write_movie_data(&self, id: MovieId, data_type: MovieDataType) -> Result<Self::W, Error> {
        let file_path = self.get_file_path(&id, data_type, true)?;

        let file = File::create(&file_path).map_err(|e| {
            Error::Internal(format!(
                "Failed to create file '{}': {}",
                file_path.display(),
                e
            ))
        })?;

        Ok(file)
    }

    fn read_movie_data(&self, id: MovieId, data_type: MovieDataType) -> Result<Self::R, Error> {
        let file_path = self.get_file_path(&id, data_type, false)?;

        let file = File::open(&file_path).map_err(|e| {
            Error::Internal(format!(
                "Failed to open file '{}': {}",
                file_path.display(),
                e
            ))
        })?;

        Ok(file)
    }

    fn remove_movie_data(&self, id: MovieId) -> Result<(), Error> {
        let movie_data_path = self.get_movie_data_path(&id);

        std::fs::remove_dir_all(&movie_data_path).map_err(|e| {
            Error::Internal(format!(
                "Failed to remove movie data directory '{}': {}",
                movie_data_path.display(),
                e
            ))
        })?;

        info!("Removed movie data directory '{}'", id);

        Ok(())
    }
}

impl FileStorage {
    /// Returns the path for all movies data for the given id.
    ///
    /// # Arguments
    /// * `id` - The movie id for which to return the path.
    fn get_movie_data_path(&self, id: &MovieId) -> PathBuf {
        let mut file_path = self.root_dir.clone();

        file_path.push(format!("{}", id));

        file_path
    }

    /// Returns the file path for the given movie id and data type.
    ///
    /// # Arguments
    /// * `id` - The movie id for which to return the file path.
    /// * `data_type` - The type of data to return the file path.
    /// * `create_dir` - Whether to create the directory if it doesn't exist.
    fn get_file_path(
        &self,
        id: &MovieId,
        data_type: MovieDataType,
        create_dir: bool,
    ) -> Result<PathBuf, Error> {
        let mut file_path = self.get_movie_data_path(id);

        // make sure the root directory exists
        if create_dir {
            create_dir_all(&file_path).map_err(|e| {
                Error::Internal(format!(
                    "Failed to create root directory '{}': {}",
                    file_path.display(),
                    e
                ))
            })?;
        }

        match data_type {
            MovieDataType::MovieData { ext } => {
                file_path.push(format!("movie.{}", ext));
            }
        }

        Ok(file_path)
    }
}

#[cfg(test)]
mod test {
    use tempdir::TempDir;

    use crate::generate_movie_id;

    use std::io::{Read, Write};

    use super::*;

    #[test]
    fn test_write_movie_data() {
        let root_dir = TempDir::new("movies-db").unwrap();

        let options = crate::Options {
            root_dir: root_dir.path().to_path_buf(),
        };

        let storage = FileStorage::new(&options).unwrap();

        let id0 = generate_movie_id();

        // write movie data to id0
        {
            let mut w = storage
                .write_movie_data(
                    id0.clone(),
                    MovieDataType::MovieData {
                        ext: "mp4".to_string(),
                    },
                )
                .unwrap();

            writeln!(w, "Hello, world!").unwrap();
        }

        // read movie data from id0
        {
            let mut r = storage
                .read_movie_data(
                    id0.clone(),
                    MovieDataType::MovieData {
                        ext: "mp4".to_string(),
                    },
                )
                .unwrap();

            let mut s = String::new();

            r.read_to_string(&mut s).unwrap();

            assert_eq!(s, "Hello, world!\n");
        }

        // remove movie data from id0
        storage.remove_movie_data(id0.clone()).unwrap();

        assert!(storage
            .read_movie_data(
                id0,
                MovieDataType::MovieData {
                    ext: "mp4".to_string(),
                }
            )
            .is_err());
    }
}
