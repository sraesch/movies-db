use crate::{Error, MovieId, Options};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use async_trait::async_trait;

/// A single entry in the movie database.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Movie {
    /// The title of the movie.
    pub title: String,

    /// An optional description of the movie.
    #[serde(default)]
    pub description: String,

    /// A list of tags associated with the movie.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A single movie entry with timestamp.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MovieDetailed {
    pub movie: Movie,
    pub movie_file_info: Option<MovieFileInfo>,
    pub screenshot_file_info: Option<ScreenshotInfo>,
    pub date: DateTime<Utc>,
}

/// The sorting order for the movies.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum SortingField {
    #[serde(rename(serialize = "title", deserialize = "title"))]
    Title,

    #[serde(rename(serialize = "date", deserialize = "date"))]
    Date,
}

impl Default for SortingField {
    fn default() -> Self {
        Self::Date
    }
}

/// The sorting order for the movies.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum SortingOrder {
    #[serde(rename(serialize = "ascending", deserialize = "ascending"))]
    Ascending,

    #[serde(rename(serialize = "descending", deserialize = "descending"))]
    Descending,
}

impl Default for SortingOrder {
    fn default() -> Self {
        Self::Descending
    }
}

/// The file info for a stored movie file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MovieFileInfo {
    /// the extension of the movie file in lower case, e.g., "mp4"
    pub extension: String,

    // the mime type of the movie file, e.g., "video/mp4"
    pub mime_type: String,
}

/// The screenshot info for a stored movie file.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScreenshotInfo {
    /// the extension of the screenshot file in lower case, e.g., "png"
    pub extension: String,

    // the mime type of the screenshot file, e.g., "image/png"
    pub mime_type: String,
}

/// A query for searching movies in the database.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MovieSearchQuery {
    /// The field used for sorting
    #[serde(default)]
    pub sorting_field: SortingField,

    /// The order used for sorting
    #[serde(default)]
    pub sorting_order: SortingOrder,

    /// Optionally, a search string for the title of the movie. If provided, only movies whose
    /// title matches the search string will be returned.
    /// Wildcards are supported, e.g., *foo* will match any movie whose title contains "foo".
    pub title: Option<String>,

    /// A sorted list of lower case tags that must match the movie.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Optionally, the start index of the movies to return.
    pub start_index: Option<usize>,

    /// Optionally, the maximal number of results to return.
    pub num_results: Option<usize>,
}

/// The movies index manages a list of all movies in the database.
/// Additionally, it provides methods for managing and searching movies.
#[async_trait]
pub trait MoviesIndex: Send + Sync {
    /// Creates a new instance of the movies index
    ///
    /// # Arguments
    /// `options` - The options for the service
    fn new(options: &Options) -> Result<Self, Error>
    where
        Self: Sized;

    /// Adds a new movie to the index.
    ///
    /// # Arguments
    /// `movie` - The movie to add to the index
    async fn add_movie(&mut self, movie: Movie) -> Result<MovieId, Error>;

    /// Returns the the movie for the given ID.
    ///
    /// # Arguments
    /// `id` - The ID of the movie to return.
    async fn get_movie(&self, id: &MovieId) -> Result<MovieDetailed, Error>;

    /// Updates the movie file info for the given ID.
    ///
    /// # Arguments
    /// `id` - The ID of the movie to update.
    /// `movie_file_info` - The new movie file info.
    async fn update_movie_file_info(
        &mut self,
        id: &MovieId,
        movie_file_info: MovieFileInfo,
    ) -> Result<(), Error>;

    /// Updates the screenshot info for the given ID.
    ///
    /// # Arguments
    /// `id` - The ID of the movie to update.
    /// `screenshot_info` - The new screenshot info.
    async fn update_screenshot_info(
        &mut self,
        id: &MovieId,
        screenshot_info: ScreenshotInfo,
    ) -> Result<(), Error>;

    /// Removes the movie for the given ID.
    ///
    /// # Arguments
    /// `id` - The ID of the movie to remove.
    async fn remove_movie(&mut self, id: &MovieId) -> Result<(), Error>;

    /// Searches the movies index for movies matching the given query.
    ///
    /// # Arguments
    /// `query` - The query to search for.
    async fn search_movies(&self, query: MovieSearchQuery) -> Result<Vec<MovieId>, Error>;

    /// Returns a list of all tags with the number of movies associated with each tag.
    async fn get_tag_list_with_count(&self) -> Result<Vec<(String, usize)>, Error>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_query_serialization() {
        let query_string = r#"
            {
                "sorting_field": "title",
                "sorting_order": "ascending",
                "title": "foo",
                "tags": ["bar", "baz"]
            }
        "#;

        let query = serde_json::from_str::<MovieSearchQuery>(query_string).unwrap();

        assert_eq!(query.title, Some("foo".to_string()));
        assert_eq!(query.tags, vec!["bar".to_string(), "baz".to_string()]);
        assert_eq!(query.sorting_field, SortingField::Title);
        assert_eq!(query.sorting_order, SortingOrder::Ascending);

        let query_string = r#"
            {
                "sorting_field": "date",
                "sorting_order": "descending",
                "title": "foo"
            }
        "#;

        let query = serde_json::from_str::<MovieSearchQuery>(query_string).unwrap();

        assert_eq!(query.title, Some("foo".to_string()));
        assert!(query.tags.is_empty());
        assert_eq!(query.sorting_field, SortingField::Date);
        assert_eq!(query.sorting_order, SortingOrder::Descending);
    }
}
