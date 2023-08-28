use std::fs::create_dir_all;

use chrono::{DateTime, Utc};
use itertools::Itertools;
use log::{debug, error, info};
use rusqlite::{Connection, Result};
use tokio::sync::Mutex;

use async_trait::async_trait;

use crate::{
    generate_movie_id, Error, Movie, MovieDetailed, MovieFileInfo, MovieId, MovieSearchQuery,
    MoviesIndex, Options, ScreenshotInfo, SortingField, SortingOrder,
};

pub struct SqliteMoviesIndex {
    /// The connection to the database.
    connection: Mutex<Connection>,
}

impl SqliteMoviesIndex {
    fn create_tables(connection: &Connection) -> Result<(), rusqlite::Error> {
        info!("Create the tables...");
        connection.execute(
            "CREATE TABLE IF NOT EXISTS movies (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                date_added TEXT NOT NULL
            )",
            (),
        )?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id TEXT NOT NULL,
                tag TEXT NOT NULL
            )",
            (),
        )?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS file_infos (
                id TEXT PRIMARY KEY,
                mime_type TEXT NOT NULL,
                extension TEXT NOT NULL
            )",
            (),
        )?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS screenshot_infos (
                id TEXT PRIMARY KEY,
                mime_type TEXT NOT NULL,
                extension TEXT NOT NULL
            )",
            (),
        )?;

        info!("Create the tables...DONE");

        Ok(())
    }

    async fn search_movies_impl(&self, query: MovieSearchQuery) -> Result<Vec<MovieId>, Error> {
        let query_string = if query.tags.is_empty() {
            self.create_search_movies_no_tags_query_string(&query)
        } else {
            self.create_search_movies_with_tags_query_string(&query)
        };

        let connection = self.connection.lock().await;

        let mut stmt = connection.prepare(&query_string)?;

        let rows = stmt.query_map([], |row| {
            let id: MovieId = row.get(0)?;

            Ok(id)
        })?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row?);
        }

        Ok(ids)
    }

    fn create_search_movies_no_tags_query_string(&self, query: &MovieSearchQuery) -> String {
        // search query without tags:
        // SELECT m.id FROM movies m WHERE m.title LIKE '%Das%' ORDER BY title DESC LIMIT 10 OFFSET 0

        let mut query_string = "SELECT m.id FROM movies m".to_owned();

        // check for where clause
        match &query.title {
            Some(title) => {
                let title = title.replace('*', "%");

                query_string.push_str(" WHERE m.title LIKE '");
                query_string.push_str(&title);
                query_string.push('\'');
            }
            None => {}
        }

        query_string.push_str(&Self::create_order_and_limit_string(query));

        query_string
    }

    fn create_search_movies_with_tags_query_string(&self, query: &MovieSearchQuery) -> String {
        // search query without tags:
        // SELECT m.id FROM movies m, tags t WHERE m.title LIKE '%Das%' AND m.id = t.id AND t.tag IN ('war','germany') GROUP BY m.id ORDER BY title DESC LIMIT 10 OFFSET 0
        let mut query_string = "SELECT m.id FROM movies m, tags t WHERE m.id = t.id ".to_owned();

        // create WHERE clause for the tags
        let tags = query
            .tags
            .iter()
            .map(|tag| format!("'{}'", tag.to_lowercase()))
            .join(",");
        let tags = "(".to_owned() + &tags + ")";
        query_string.push_str(" AND t.tag IN ");
        query_string.push_str(&tags);

        // add WHERE clause for the title if available
        match &query.title {
            Some(title) => {
                let title = title.replace('*', "%");

                query_string.push_str(" AND m.title LIKE '");
                query_string.push_str(&title);
                query_string.push('\'');
            }
            None => {}
        }

        // add GROUP BY clause
        query_string.push_str(" GROUP BY m.id ");

        query_string.push_str(&Self::create_order_and_limit_string(query));

        query_string
    }

    /// Creates the ORDER and LIMIT string based on the provided query.
    ///
    /// # Arguments
    /// * `query` - The query to create the ORDER and LIMIT string for.
    fn create_order_and_limit_string(query: &MovieSearchQuery) -> String {
        let mut order_and_limit = String::new();

        // field
        let field = match query.sorting_field {
            SortingField::Title => "m.title",
            SortingField::Date => "m.date_added",
        };

        // order
        let order = match query.sorting_order {
            SortingOrder::Ascending => "ASC",
            SortingOrder::Descending => "DESC",
        };

        order_and_limit.push_str(&format!(" ORDER BY {} {} ", field, order));

        // limit
        if let Some(limit) = query.num_results {
            order_and_limit.push_str(&format!(" LIMIT {} ", limit));
        }

        // offset
        if let Some(offset) = query.start_index {
            order_and_limit.push_str(&format!(" OFFSET {} ", offset));
        }

        order_and_limit
    }
}

#[async_trait]
impl MoviesIndex for SqliteMoviesIndex {
    fn new(options: &Options) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Err(err) = create_dir_all(&options.root_dir) {
            error!("Failed to create the root directory: {}", err);
            return Err(err.into());
        }

        let mut sqlite_path = options.root_dir.clone();
        sqlite_path.push("movies.db");

        debug!("SQLite database path: {}", sqlite_path.display());

        if sqlite_path.exists() {
            info!("Found existing movies.db");
        }

        match Connection::open(sqlite_path) {
            Err(err) => {
                error!("Failed to open the SQLite database: {}", err);
                Err(Error::IO(format!("Failed to open SQLite DB{}", err)))
            }
            Ok(connection) => {
                if let Err(err) = Self::create_tables(&connection) {
                    error!("Failed to create the tables: {}", err);
                    return Err(Error::Internal(format!(
                        "Failed to create the tables: {}",
                        err
                    )));
                }

                let connection = Mutex::new(connection);

                Ok(Self { connection })
            }
        }
    }

    async fn add_movie(&mut self, movie: Movie) -> Result<MovieId, Error> {
        let id = generate_movie_id();
        info!("Adding movie {} with id {}", movie.title, id);

        // check if movie has title
        if movie.title.is_empty() {
            error!("Movie has no title");
            return Err(Error::InvalidArgument(
                "Movie title must not be empty".to_string(),
            ));
        }

        let date = chrono::Utc::now().to_rfc3339();

        let connection = self.connection.lock().await;

        // insert movie details
        connection.execute(
            "INSERT INTO movies (id, title, description, date_added) VALUES (?1, ?2, ?3, ?4)",
            (&id, &movie.title, &movie.description, &date),
        )?;

        // insert tags
        let mut stmt = connection.prepare("INSERT INTO tags (id, tag) VALUES (?1, ?2)")?;
        for tag in movie.tags {
            stmt.execute((&id, &tag.to_lowercase()))?;
        }

        Ok(id)
    }

    async fn get_movie(&self, id: &MovieId) -> Result<MovieDetailed, Error> {
        info!("Getting movie with id {}", id);

        let connection = self.connection.lock().await;

        // get the movie details
        let mut stmt =
            connection.prepare("SELECT title, description, date_added FROM movies WHERE id=:id")?;
        let mut rows = stmt.query_map(&[(":id", &id)], |row| {
            let title: String = row.get(0)?;
            let description: String = row.get(1)?;
            let date: String = row.get(2)?;

            Ok((title, description, date))
        })?;

        let row = match rows.next() {
            None => {
                error!("No movie with id {} found", id);
                return Err(Error::NotFound(format!("No movie with id {} found", id)));
            }
            Some(row) => row?,
        };

        let title = row.0;
        let description = row.1;
        let date_added: DateTime<Utc> = match row.2.parse() {
            Err(err) => {
                error!("Failed to parse date: {}", err);
                return Err(Error::Internal(format!("Failed to parse date: {}", err)));
            }
            Ok(date) => date,
        };

        // get the tags
        let mut stmt = connection.prepare("SELECT tag FROM tags WHERE id=:id ORDER BY tag")?;
        let rows = stmt.query_map(&[(":id", &id)], |row| {
            let tag: String = row.get(0)?;

            Ok(tag)
        })?;

        let mut tags: Vec<String> = Vec::new();
        for row in rows {
            tags.push(row?);
        }

        // get movie file info, if available
        let mut stmt =
            connection.prepare("SELECT mime_type, extension FROM file_infos WHERE id=:id")?;
        let mut rows = stmt.query_map(&[(":id", &id)], |row| {
            let mime_type: String = row.get(0)?;
            let extension: String = row.get(1)?;

            Ok((mime_type, extension))
        })?;

        let movie_file_info = match rows.next() {
            None => None,
            Some(row) => {
                let (mime_type, extension) = row?;

                Some(MovieFileInfo {
                    mime_type,
                    extension,
                })
            }
        };

        // get movie screenshot info, if available
        let mut stmt =
            connection.prepare("SELECT mime_type, extension FROM screenshot_infos WHERE id=:id")?;
        let mut rows = stmt.query_map(&[(":id", &id)], |row| {
            let mime_type: String = row.get(0)?;
            let extension: String = row.get(1)?;

            Ok((mime_type, extension))
        })?;

        let screenshot_file_info = match rows.next() {
            None => None,
            Some(row) => {
                let (mime_type, extension) = row?;

                Some(ScreenshotInfo {
                    mime_type,
                    extension,
                })
            }
        };

        let movie = Movie {
            title,
            description,
            tags,
        };

        Ok(MovieDetailed {
            movie,
            date: date_added,
            movie_file_info,
            screenshot_file_info,
        })
    }

    async fn remove_movie(&mut self, id: &MovieId) -> Result<(), Error> {
        let connection = self.connection.lock().await;

        // delete movie details, stop if there was no movie with the given id
        if connection.execute("DELETE FROM movies WHERE id=:id", &[(":id", &id)])? == 0 {
            error!("No movie with id {} found", id);
            return Err(Error::NotFound(format!("No movie with id {} found", id)));
        }

        // delete tags
        connection.execute("DELETE FROM tags WHERE id=:id", &[(":id", &id)])?;

        // delete file info
        connection.execute("DELETE FROM file_infos WHERE id=:id", &[(":id", &id)])?;

        // delete screenshot info
        connection.execute("DELETE FROM screenshot_infos WHERE id=:id", &[(":id", &id)])?;

        Ok(())
    }

    async fn update_movie_file_info(
        &mut self,
        id: &MovieId,
        movie_file_info: MovieFileInfo,
    ) -> Result<(), Error> {
        let connection = self.connection.lock().await;

        connection.execute(
            "INSERT OR REPLACE INTO file_infos (id, mime_type, extension) VALUES (?1, ?2, ?3)",
            (&id, &movie_file_info.mime_type, &movie_file_info.extension),
        )?;

        Ok(())
    }

    async fn update_screenshot_info(
        &mut self,
        id: &MovieId,
        screenshot_info: ScreenshotInfo,
    ) -> Result<(), Error> {
        let connection = self.connection.lock().await;

        connection.execute(
            "INSERT OR REPLACE INTO screenshot_infos (id, mime_type, extension) VALUES (?1, ?2, ?3)",
            (&id, &screenshot_info.mime_type, &screenshot_info.extension),
        )?;

        Ok(())
    }

    async fn search_movies(&self, query: MovieSearchQuery) -> Result<Vec<MovieId>, Error> {
        self.search_movies_impl(query).await
    }

    async fn get_tag_list_with_count(&self) -> Result<Vec<(String, usize)>, Error> {
        let connection = self.connection.lock().await;

        let mut stmt = connection.prepare(
            "SELECT tag, COUNT(*) FROM tags GROUP BY tag ORDER BY COUNT(*) DESC, tag ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            let tag: String = row.get(0)?;
            let count: usize = row.get(1)?;

            Ok((tag, count))
        })?;

        let mut tags: Vec<(String, usize)> = Vec::new();
        for row in rows {
            tags.push(row?);
        }

        Ok(tags)
    }
}

#[cfg(test)]
mod test {
    use tempdir::TempDir;

    use crate::Movie;

    use super::*;

    fn create_test_movies() -> Vec<Movie> {
        vec![Movie { title: "Doctor Who".to_owned(), description: "The Doctor, a Time Lord from the race whose home planet is Gallifrey, travels through time and space in their ship the TARDIS (an acronym for Time and Relative Dimension In Space) with numerous companions.".to_owned(), tags: vec!["Sci-Fi".to_owned(), "time travel".to_owned(), "british".to_owned(), "tv show".to_owned()] },
        Movie { title: "The X-Files".to_owned(), description: "Two F.B.I. Agents, Fox Mulder the believer and Dana Scully the skeptic, investigate the strange and unexplained, while hidden forces work to impede their efforts.".to_owned(), tags: vec!["crime".to_owned(), "drama".to_owned(), "mystery".to_owned(), "usa".to_owned(), "tv show".to_owned()] },
        Movie { title: "E.T. the Extra-Terrestrial".to_owned(), description: "A troubled child summons the courage to help a friendly alien escape from Earth and return to his home planet.".to_owned(), tags: vec!["adventure".to_owned(), "family".to_owned(), "Sci-Fi".to_owned(), "usa".to_owned(), "movie".to_owned()] },
        Movie { title: "Das Boot".to_owned(), description: "A German U-boat stalks the frigid waters of the North Atlantic as its young crew experience the sheer terror and claustrophobic life of a submariner in World War II.".to_owned(), tags: vec!["drama".to_owned(), "war".to_owned(), "germany".to_owned(), "movie".to_owned()] }]
    }

    #[tokio::test]
    async fn test_add_movie() {
        let root_dir = TempDir::new("movies-db").unwrap();
        let mut options = Options::default();
        options.root_dir = root_dir.path().to_path_buf();
        let mut index = SqliteMoviesIndex::new(&options).unwrap();

        let movies = create_test_movies();

        for movie in movies {
            let ret = index.add_movie(movie.clone()).await;
            assert!(ret.is_ok());
        }
    }

    #[tokio::test]
    async fn test_get_movie() {
        let root_dir = TempDir::new("movies-db").unwrap();
        let mut options = Options::default();
        options.root_dir = root_dir.path().to_path_buf();
        let mut index = SqliteMoviesIndex::new(&options).unwrap();

        let movies = create_test_movies();

        let mut movie_ids: Vec<MovieId> = Vec::with_capacity(movies.len());
        for movie in movies.iter() {
            let id = index.add_movie(movie.clone()).await.unwrap();
            movie_ids.push(id);
        }

        for (id, movie) in movie_ids.iter().zip(movies.iter()) {
            let db_movie = index.get_movie(id).await;
            assert!(db_movie.is_ok());
            let db_movie = db_movie.unwrap();

            assert_eq!(movie.title, db_movie.movie.title);
            assert_eq!(movie.description, db_movie.movie.description);

            let mut ref_tags = movie.tags.clone();
            ref_tags
                .iter_mut()
                .for_each(|tag| *tag = tag.to_lowercase());
            ref_tags.sort();
            assert_eq!(ref_tags, db_movie.movie.tags);
        }
    }

    #[tokio::test]
    async fn test_remove_movie() {
        let root_dir = TempDir::new("movies-db").unwrap();
        let mut options = Options::default();
        options.root_dir = root_dir.path().to_path_buf();
        let mut index = SqliteMoviesIndex::new(&options).unwrap();

        let movies = create_test_movies();

        let mut movie_ids: Vec<MovieId> = Vec::with_capacity(movies.len());
        for movie in movies.iter() {
            let id = index.add_movie(movie.clone()).await.unwrap();
            movie_ids.push(id);
        }

        movie_ids.sort();

        index.remove_movie(&movie_ids[0]).await.unwrap();
        assert!(index.remove_movie(&movie_ids[0]).await.is_err());

        let movie_ids = &movie_ids[1..];
        let mut listed_movie_ids = index.search_movies(Default::default()).await.unwrap();
        listed_movie_ids.sort();

        assert_eq!(movie_ids, listed_movie_ids);
    }

    async fn movie_ids_to_titles(index: &SqliteMoviesIndex, movie_ids: &[MovieId]) -> Vec<String> {
        let mut movie_titles: Vec<String> = Vec::with_capacity(movie_ids.len());

        for movie_id in movie_ids {
            let movie = index.get_movie(movie_id).await.unwrap();
            movie_titles.push(movie.movie.title.clone());
        }

        movie_titles
    }

    #[tokio::test]
    async fn test_query_movies() {
        let root_dir = TempDir::new("movies-db").unwrap();
        let mut options = Options::default();
        options.root_dir = root_dir.path().to_path_buf();
        let mut index = SqliteMoviesIndex::new(&options).unwrap();

        let movies = create_test_movies();

        for movie in movies.iter() {
            let ret = index.add_movie(movie.clone()).await;
            assert!(ret.is_ok());
        }

        // test query 1A: Search all movies (ascending order by title)
        let mut query: MovieSearchQuery = Default::default();
        query.sorting_field = SortingField::Title;
        query.sorting_order = SortingOrder::Ascending;
        let movie_title: Vec<String> =
            movie_ids_to_titles(&index, &index.search_movies(query).await.unwrap()).await;

        assert_eq!(
            movie_title,
            vec![
                "Das Boot",
                "Doctor Who",
                "E.T. the Extra-Terrestrial",
                "The X-Files",
            ]
        );

        // test query 1B: Search all movies (descending order by title)
        let mut query: MovieSearchQuery = Default::default();
        query.sorting_field = SortingField::Title;
        query.sorting_order = SortingOrder::Descending;
        let movie_title: Vec<String> =
            movie_ids_to_titles(&index, &index.search_movies(query).await.unwrap()).await;

        assert_eq!(
            movie_title,
            vec![
                "The X-Files",
                "E.T. the Extra-Terrestrial",
                "Doctor Who",
                "Das Boot",
            ]
        );

        // // test query 2: Search only science fiction movies
        let mut query: MovieSearchQuery = Default::default();
        query.tags = vec!["Sci-Fi".to_owned()];
        query.sorting_field = SortingField::Title;
        query.sorting_order = SortingOrder::Ascending;
        let search_result = index.search_movies(query).await.unwrap();
        let title_list: Vec<String> = movie_ids_to_titles(&index, &search_result).await;
        assert_eq!(
            title_list,
            vec![
                "Doctor Who".to_owned(),
                "E.T. the Extra-Terrestrial".to_owned()
            ]
        );

        // test query 3: Search 'Das Boot'
        let query = MovieSearchQuery {
            sorting_field: Default::default(),
            sorting_order: Default::default(),
            title: Some("Boot".to_owned()),
            tags: vec![],
            start_index: None,
            num_results: None,
        };
        assert_eq!(index.search_movies(query).await.unwrap().len(), 0);
        let query = MovieSearchQuery {
            sorting_field: Default::default(),
            sorting_order: Default::default(),
            title: Some("*Boot".to_owned()),
            tags: vec![],
            start_index: None,
            num_results: None,
        };
        assert_eq!(
            movie_ids_to_titles(&index, &index.search_movies(query).await.unwrap()).await,
            ["Das Boot"]
        );

        // test query 4: Limited ranges
        let query = MovieSearchQuery {
            sorting_field: SortingField::Title,
            sorting_order: SortingOrder::Ascending,
            title: None,
            tags: vec![],
            start_index: Some(0),
            num_results: Some(1),
        };
        assert_eq!(
            movie_ids_to_titles(&index, &index.search_movies(query).await.unwrap()).await,
            ["Das Boot"]
        );
        let query = MovieSearchQuery {
            sorting_field: SortingField::Title,
            sorting_order: SortingOrder::Ascending,
            title: None,
            tags: vec![],
            start_index: Some(1),
            num_results: Some(2),
        };
        assert_eq!(
            movie_ids_to_titles(&index, &index.search_movies(query).await.unwrap()).await,
            ["Doctor Who", "E.T. the Extra-Terrestrial"]
        );
    }

    #[tokio::test]
    async fn test_add_movie_file_info() {
        let root_dir = TempDir::new("movies-db").unwrap();
        let mut options = Options::default();
        options.root_dir = root_dir.path().to_path_buf();
        let mut index = SqliteMoviesIndex::new(&options).unwrap();
        let movies = create_test_movies();

        let mut movie_ids: Vec<MovieId> = Vec::with_capacity(movies.len());
        for movie in movies.iter() {
            let ret = index.add_movie(movie.clone()).await;
            assert!(ret.is_ok());

            movie_ids.push(ret.unwrap());
        }

        // make sure non of the added movies has a file info
        for movie_id in movie_ids.iter() {
            let movie = index.get_movie(&movie_id).await.unwrap();
            assert!(movie.movie_file_info.is_none());
        }

        // add movie file info only to the first two movies
        let (left_movie_ids, right_movie_ids) = movie_ids.split_at(2);
        assert_eq!(left_movie_ids.len(), 2);
        let movie_file_infos = [
            MovieFileInfo {
                extension: ".mp4".to_owned(),
                mime_type: "video/mp4".to_owned(),
            },
            MovieFileInfo {
                extension: ".wmv".to_owned(),
                mime_type: "video/x-ms-wmv".to_owned(),
            },
        ];

        for (movie_id, movie_file_info) in left_movie_ids.iter().zip(movie_file_infos.iter()) {
            index
                .update_movie_file_info(movie_id, movie_file_info.clone())
                .await
                .unwrap();
        }

        // check that the movie file info was added to the first two movies
        for (movie_id, movie_file_info) in left_movie_ids.iter().zip(movie_file_infos.iter()) {
            let movie = index.get_movie(movie_id).await.unwrap();
            assert_eq!(movie.movie_file_info, Some(movie_file_info.clone()));
        }

        // check that the movie file info was not added to the other movies
        for movie_id in right_movie_ids.iter() {
            let movie = index.get_movie(movie_id).await.unwrap();
            assert!(movie.movie_file_info.is_none());
        }
    }

    #[tokio::test]
    async fn test_add_screenshot_info() {
        let root_dir = TempDir::new("movies-db").unwrap();
        let mut options = Options::default();
        options.root_dir = root_dir.path().to_path_buf();
        let mut index = SqliteMoviesIndex::new(&options).unwrap();
        let movies = create_test_movies();

        let mut movie_ids: Vec<MovieId> = Vec::with_capacity(movies.len());
        for movie in movies.iter() {
            let ret = index.add_movie(movie.clone()).await;
            assert!(ret.is_ok());

            movie_ids.push(ret.unwrap());
        }

        // make sure non of the added movies has a screenshot info
        for movie_id in movie_ids.iter() {
            let movie = index.get_movie(&movie_id).await.unwrap();
            assert!(movie.screenshot_file_info.is_none());
        }

        // add screenshot info only to the first two movies
        let (left_movie_ids, right_movie_ids) = movie_ids.split_at(2);
        assert_eq!(left_movie_ids.len(), 2);
        let screenshot_infos = [
            ScreenshotInfo {
                extension: ".png".to_owned(),
                mime_type: "video/png".to_owned(),
            },
            ScreenshotInfo {
                extension: ".jpeg".to_owned(),
                mime_type: "image/jpeg".to_owned(),
            },
        ];

        for (movie_id, screenshot_info) in left_movie_ids.iter().zip(screenshot_infos.iter()) {
            index
                .update_screenshot_info(movie_id, screenshot_info.clone())
                .await
                .unwrap();
        }

        // check that the movie file info was added to the first two movies
        for (movie_id, screenshot_info) in left_movie_ids.iter().zip(screenshot_infos.iter()) {
            let movie: MovieDetailed = index.get_movie(movie_id).await.unwrap();
            assert_eq!(movie.screenshot_file_info, Some(screenshot_info.clone()));
        }

        // check that the movie file info was not added to the other movies
        for movie_id in right_movie_ids.iter() {
            let movie = index.get_movie(movie_id).await.unwrap();
            assert!(movie.screenshot_file_info.is_none());
        }
    }
}
