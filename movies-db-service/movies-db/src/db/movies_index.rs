use std::ops::Range;

use crate::{Error, MovieId, Options};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single entry in the movie database.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Movie {
    /// The title of the movie.
    pub title: String,

    /// An optional description of the movie.
    pub description: String,

    /// A list of tags associated with the movie.
    pub tags: Vec<String>,
}

/// A single movie entry with timestamp.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MovieWithDate {
    pub movie: Movie,
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

/// The sorting order for the movies.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum SortingOrder {
    #[serde(rename(serialize = "ascending", deserialize = "ascending"))]
    Ascending,

    #[serde(rename(serialize = "descending", deserialize = "descending"))]
    Descending,
}

/// A sorting for the movies.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MovieSorting {
    pub field: SortingField,
    pub order: SortingOrder,
}

impl Default for MovieSorting {
    fn default() -> Self {
        Self {
            field: SortingField::Date,
            order: SortingOrder::Descending,
        }
    }
}

/// A query for searching movies in the database.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MovieSearchQuery {
    /// The sorting being used for the movies.
    pub sorting: MovieSorting,

    /// Optionally, a search string for the title of the movie. If provided, only movies whose
    /// title matches the search string will be returned.
    /// Wildcards are supported, e.g., *foo* will match any movie whose title contains "foo".
    pub title: Option<String>,

    /// A sorted list of lower case tags that must match the movie.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Optionally, a range for the items to return can be specified.
    pub range: Option<Range<usize>>,
}

/// The movies index manages a list of all movies in the database.
/// Additionally, it provides methods for managing and searching movies.
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
    fn add_movie(&mut self, movie: Movie) -> Result<MovieId, Error>;

    /// Returns the the movie for the given ID.
    ///
    /// # Arguments
    /// `id` - The ID of the movie to return.
    fn get_movie(&self, id: &MovieId) -> Result<MovieWithDate, Error>;

    /// Removes the movie for the given ID.
    ///
    /// # Arguments
    /// `id` - The ID of the movie to remove.
    fn remove_movie(&mut self, id: &MovieId) -> Result<(), Error>;

    /// Changes the description of the movie for the given ID.
    ///
    /// # Arguments
    /// `id` - The ID of the movie to change.
    /// `description` - The new description of the movie.
    fn change_movie_description(&mut self, id: &MovieId, description: String) -> Result<(), Error>;

    /// Changes the title of the movie for the given ID.
    ///
    /// # Arguments
    /// `id` - The ID of the movie to change.
    /// `title` - The new title of the movie.
    fn change_movie_title(&mut self, id: &MovieId, title: String) -> Result<(), Error>;

    /// Changes the tags of the movie for the given ID.
    ///
    /// # Arguments
    /// `id` - The ID of the movie to change.
    /// `tags` - The new tags of the movie.
    fn change_movie_tags(&mut self, id: &MovieId, tags: Vec<String>) -> Result<(), Error>;

    /// Searches the movies index for movies matching the given query.
    ///
    /// # Arguments
    /// `query` - The query to search for.
    fn search_movies(&self, query: MovieSearchQuery) -> Result<Vec<MovieId>, Error>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_query_serialization() {
        let query_string = r#"
            {
                "sorting": {
                    "field": "title",
                    "order": "ascending"
                },
                "title": "foo",
                "tags": ["bar", "baz"]
            }
        "#;

        let query = serde_json::from_str::<MovieSearchQuery>(query_string).unwrap();

        assert_eq!(query.title, Some("foo".to_string()));
        assert_eq!(query.tags, vec!["bar".to_string(), "baz".to_string()]);
        assert_eq!(query.sorting.field, SortingField::Title);
        assert_eq!(query.sorting.order, SortingOrder::Ascending);

        let query_string = r#"
            {
                "sorting": {
                    "field": "date",
                    "order": "descending"
                },
                "title": "foo"
            }
        "#;

        let query = serde_json::from_str::<MovieSearchQuery>(query_string).unwrap();

        assert_eq!(query.title, Some("foo".to_string()));
        assert!(query.tags.is_empty());
        assert_eq!(query.sorting.field, SortingField::Date);
        assert_eq!(query.sorting.order, SortingOrder::Descending);
    }
}
