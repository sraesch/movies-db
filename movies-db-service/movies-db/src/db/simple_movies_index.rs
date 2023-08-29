use std::collections::HashMap;

use chrono::DateTime;
use log::{error, info};
use wildmatch::WildMatch;

use async_trait::async_trait;

use crate::{
    generate_movie_id, Error, Movie, MovieDetailed, MovieFileInfo, MovieId, MovieSearchQuery,
    MoviesIndex, Options, ScreenshotInfo, SortingField, SortingOrder,
};

/// A very simple and naive in-memory implementation of the movies index.
pub struct SimpleMoviesIndex {
    movies: HashMap<MovieId, MovieDetailed>,
}

impl SimpleMoviesIndex {
    /// Processes the given tags by converting them to lower case and sorting them.
    ///
    /// # Arguments
    /// `tags` - The tags to process.
    fn process_tags(tags: &mut [String]) {
        tags.iter_mut().for_each(|tag| *tag = tag.to_lowercase());
        tags.sort();
    }
}

#[async_trait]
impl MoviesIndex for SimpleMoviesIndex {
    fn new(_: &Options) -> Result<Self, Error> {
        Ok(Self {
            movies: HashMap::new(),
        })
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

        assert!(
            self.movies.get(&id).is_none(),
            "Movie with id {} already exists",
            id
        );

        let mut movie_with_date = MovieDetailed {
            movie,
            movie_file_info: None,
            screenshot_file_info: None,
            date: chrono::Utc::now(),
        };
        Self::process_tags(&mut movie_with_date.movie.tags);

        self.movies.insert(id.clone(), movie_with_date);

        Ok(id)
    }

    async fn get_movie(&self, id: &MovieId) -> Result<MovieDetailed, Error> {
        info!("Getting movie with id {}", id);

        match self.movies.get(id) {
            Some(movie) => Ok(movie.clone()),
            None => {
                error!("Movie with id {} not found", id);
                Err(Error::NotFound(format!("Movie with id {} not found", id)))
            }
        }
    }

    async fn update_movie_file_info(
        &mut self,
        id: &MovieId,
        movie_file_info: MovieFileInfo,
    ) -> Result<(), Error> {
        info!("Updating movie file info for movie with id {}", id);

        match self.movies.get_mut(id) {
            Some(movie) => {
                movie.movie_file_info = Some(movie_file_info);
                Ok(())
            }
            None => {
                error!("Movie with id {} not found", id);
                Err(Error::NotFound(format!("Movie with id {} not found", id)))
            }
        }
    }

    async fn update_screenshot_info(
        &mut self,
        id: &MovieId,
        screenshot_info: ScreenshotInfo,
    ) -> Result<(), Error> {
        info!("Updating screenshot info for movie with id {}", id);

        match self.movies.get_mut(id) {
            Some(movie) => {
                movie.screenshot_file_info = Some(screenshot_info);
                Ok(())
            }
            None => {
                error!("Movie with id {} not found", id);
                Err(Error::NotFound(format!("Movie with id {} not found", id)))
            }
        }
    }

    async fn remove_movie(&mut self, id: &MovieId) -> Result<(), Error> {
        info!("Removing movie with id {}", id);

        match self.movies.remove(id) {
            Some(_) => Ok(()),
            None => {
                error!("Movie with id {} not found", id);
                Err(Error::NotFound(format!("Movie with id {} not found", id)))
            }
        }
    }

    async fn search_movies(&self, mut query: MovieSearchQuery) -> Result<Vec<MovieId>, Error> {
        info!("Searching movies with query {:?}", query);
        Self::process_tags(&mut query.tags);

        // get sorted movie ids
        let in_movie_ids = self.get_movies_sorted(query.sorting_field, query.sorting_order);

        // create wildcard query if provided
        let title_query: Option<WildMatch> = query.title.map(|s| WildMatch::new(&s));

        let start_index = query.start_index.unwrap_or(0);
        let end_index = match query.num_results {
            Some(num_results) => start_index + num_results,
            None => usize::MAX,
        };

        let mut num_hits = 0usize;
        let mut movie_ids = Vec::new();
        for id in in_movie_ids.iter() {
            let movie_with_date = match self.movies.get(id) {
                Some(movie_with_date) => movie_with_date,
                None => {
                    error!("Movie with id {} not found", id);
                    return Err(Error::Internal(format!("Movie with id {} not found", id)));
                }
            };

            let movie = &movie_with_date.movie;

            // if a title query is available and the movie title does not match, skip
            if let Some(ref title_query) = title_query {
                if !title_query.matches(&movie.title) {
                    continue;
                }
            }

            // check that all tags match
            if !query
                .tags
                .iter()
                .all(|tag| movie.tags.binary_search(tag).is_ok())
            {
                continue;
            }

            // add movie id if index is within range
            if num_hits >= start_index && end_index > num_hits {
                movie_ids.push(id.clone());
            } else if num_hits >= end_index {
                break;
            }

            num_hits += 1;
        }

        Ok(movie_ids)
    }

    async fn get_tag_list_with_count(&self) -> Result<Vec<(String, usize)>, Error> {
        info!("Getting tag list with count");

        let mut tag_map: HashMap<String, usize> = HashMap::new();

        for movie in self.movies.values() {
            for tag in movie.movie.tags.iter() {
                let count = tag_map.entry(tag.clone()).or_insert(0);
                *count += 1;
            }
        }

        let mut tag_list: Vec<(String, usize)> = tag_map.into_iter().collect();
        tag_list.sort_unstable_by(|(_, lhs), (_, rhs)| rhs.cmp(lhs));

        Ok(tag_list)
    }
}

impl SimpleMoviesIndex {
    /// Returns a list of all movies sorted according to the given sorting parameter.
    ///
    /// # Arguments
    /// * `field` - The field by which the movies should be sorted.
    /// * `order` - The order in which the movies should be sorted.
    fn get_movies_sorted(&self, field: SortingField, order: SortingOrder) -> Vec<MovieId> {
        match field {
            SortingField::Title => self.get_movies_sorted_by_title(order),
            SortingField::Date => self.get_movies_sorted_by_date(order),
        }
    }

    /// Returns a list of all movies sorted by their title.
    ///
    /// # Arguments
    /// * `order` - The order in which the movies should be sorted.
    fn get_movies_sorted_by_title(&self, order: SortingOrder) -> Vec<MovieId> {
        let mut movies: Vec<(MovieId, String)> = self
            .movies
            .iter()
            .map(|(id, movie)| (id.clone(), movie.movie.title.clone()))
            .collect();

        movies.sort_unstable_by(|(_, lhs), (_, rhs)| lhs.cmp(rhs));

        if order == SortingOrder::Ascending {
            movies.iter().map(|(id, _)| id.clone()).collect()
        } else {
            movies.iter().rev().map(|(id, _)| id.clone()).collect()
        }
    }

    fn get_movies_sorted_by_date(&self, order: SortingOrder) -> Vec<MovieId> {
        let mut movies: Vec<(MovieId, DateTime<_>)> = self
            .movies
            .iter()
            .map(|(id, movie)| (id.clone(), movie.date))
            .collect();

        movies.sort_unstable_by(|(_, lhs), (_, rhs)| lhs.cmp(rhs));

        if order == SortingOrder::Ascending {
            movies.iter().map(|(id, _)| id.clone()).collect()
        } else {
            movies.iter().rev().map(|(id, _)| id.clone()).collect()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_test_movies() -> Vec<Movie> {
        vec![Movie { title: "Doctor Who".to_owned(), description: "The Doctor, a Time Lord from the race whose home planet is Gallifrey, travels through time and space in their ship the TARDIS (an acronym for Time and Relative Dimension In Space) with numerous companions.".to_owned(), tags: vec!["Sci-Fi".to_owned(), "time travel".to_owned(), "british".to_owned(), "tv show".to_owned()] },
        Movie { title: "The X-Files".to_owned(), description: "Two F.B.I. Agents, Fox Mulder the believer and Dana Scully the skeptic, investigate the strange and unexplained, while hidden forces work to impede their efforts.".to_owned(), tags: vec!["crime".to_owned(), "drama".to_owned(), "mystery".to_owned(), "usa".to_owned(), "tv show".to_owned()] },
        Movie { title: "E.T. the Extra-Terrestrial".to_owned(), description: "A troubled child summons the courage to help a friendly alien escape from Earth and return to his home planet.".to_owned(), tags: vec!["adventure".to_owned(), "family".to_owned(), "Sci-Fi".to_owned(), "usa".to_owned(), "movie".to_owned()] },
        Movie { title: "Das Boot".to_owned(), description: "A German U-boat stalks the frigid waters of the North Atlantic as its young crew experience the sheer terror and claustrophobic life of a submariner in World War II.".to_owned(), tags: vec!["drama".to_owned(), "war".to_owned(), "germany".to_owned(), "movie".to_owned()] }]
    }

    #[tokio::test]
    async fn test_add_movie() {
        let mut index = SimpleMoviesIndex::new(&Options::default()).unwrap();

        let movies = create_test_movies();

        for movie in movies {
            let ret = index.add_movie(movie.clone()).await;
            assert!(ret.is_ok());
        }
    }

    #[tokio::test]
    async fn test_get_movie() {
        let mut index = SimpleMoviesIndex::new(&Options::default()).unwrap();
        let movies = create_test_movies();

        let mut movie_ids: Vec<MovieId> = Vec::with_capacity(movies.len());
        for movie in movies.iter() {
            let id = index.add_movie(movie.clone()).await.unwrap();
            movie_ids.push(id);
        }

        for (id, movie) in movie_ids.iter().zip(movies.iter()) {
            let db_movie = index.get_movie(id).await.unwrap();

            assert_eq!(movie.title, db_movie.movie.title);
            assert_eq!(movie.description, db_movie.movie.description);

            let mut ref_tags = movie.tags.clone();
            SimpleMoviesIndex::process_tags(&mut ref_tags);
            assert_eq!(ref_tags, db_movie.movie.tags);
        }
    }

    #[tokio::test]
    async fn test_remove_movie() {
        let mut index = SimpleMoviesIndex::new(&Options::default()).unwrap();
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

    async fn movie_ids_to_titles(index: &SimpleMoviesIndex, movie_ids: &[MovieId]) -> Vec<String> {
        let mut movie_titles: Vec<String> = Vec::with_capacity(movie_ids.len());

        for movie_id in movie_ids {
            let movie = index.get_movie(movie_id).await.unwrap();
            movie_titles.push(movie.movie.title.clone());
        }

        movie_titles
    }

    #[tokio::test]
    async fn test_query_movies() {
        let mut index = SimpleMoviesIndex::new(&Options::default()).unwrap();
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
        let mut index = SimpleMoviesIndex::new(&Options::default()).unwrap();
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
        let mut index = SimpleMoviesIndex::new(&Options::default()).unwrap();
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
