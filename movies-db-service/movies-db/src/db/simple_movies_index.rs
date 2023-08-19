use std::{collections::HashMap, ops::Range};

use chrono::DateTime;
use log::{debug, error, info};
use wildmatch::WildMatch;

use crate::{
    generate_movie_id, Error, Movie, MovieId, MovieSearchQuery, MovieWithDate, MoviesIndex,
    Options, SortingField, SortingOrder,
};

/// A very simple and naive in-memory implementation of the movies index.
pub struct SimpleMoviesIndex {
    movies: HashMap<MovieId, MovieWithDate>,
}

impl SimpleMoviesIndex {
    /// Processes the given tags by converting them to lower case and sorting them.
    ///
    /// # Arguments
    /// `tags` - The tags to process.
    fn process_tags(tags: &mut Vec<String>) {
        tags.iter_mut().for_each(|tag| *tag = tag.to_lowercase());
        tags.sort();
    }
}

impl MoviesIndex for SimpleMoviesIndex {
    fn new(_: &Options) -> Result<Self, Error> {
        Ok(Self {
            movies: HashMap::new(),
        })
    }

    fn add_movie(&mut self, movie: Movie) -> Result<MovieId, Error> {
        let id = generate_movie_id();
        info!("Adding movie {} with id {}", movie.title, id);

        // check if movie has title
        if movie.title.is_empty() {
            error!("Movie has no title");
            return Err(Error::InvalidArgument(format!(
                "Movie title must not be empty"
            )));
        }

        assert!(
            self.movies.get(&id).is_none(),
            "Movie with id {} already exists",
            id
        );

        let mut movie_with_date = MovieWithDate {
            movie,
            date: chrono::Utc::now(),
        };
        Self::process_tags(&mut movie_with_date.movie.tags);

        self.movies.insert(id.clone(), movie_with_date);

        Ok(id)
    }

    fn get_movie(&self, id: &MovieId) -> Result<MovieWithDate, Error> {
        info!("Getting movie with id {}", id);

        match self.movies.get(id) {
            Some(movie) => Ok(movie.clone()),
            None => {
                error!("Movie with id {} not found", id);
                Err(Error::NotFound(format!("Movie with id {} not found", id)))
            }
        }
    }

    fn remove_movie(&mut self, id: &MovieId) -> Result<(), Error> {
        info!("Removing movie with id {}", id);

        match self.movies.remove(id) {
            Some(_) => Ok(()),
            None => {
                error!("Movie with id {} not found", id);
                Err(Error::NotFound(format!("Movie with id {} not found", id)))
            }
        }
    }

    fn change_movie_description(&mut self, id: &MovieId, description: String) -> Result<(), Error> {
        info!("Changing description of movie with id {}", id);
        debug!("New description: {:?}", description);

        match self.movies.get_mut(id) {
            Some(movie_with_date) => {
                movie_with_date.movie.description = description;
                Ok(())
            }
            None => {
                error!("Movie with id {} not found", id);
                Err(Error::NotFound(format!("Movie with id {} not found", id)))
            }
        }
    }

    fn change_movie_title(&mut self, id: &MovieId, title: String) -> Result<(), Error> {
        info!("Changing title of movie with id {}", id);
        debug!("New title: {:?}", title);

        match self.movies.get_mut(id) {
            Some(movie_with_date) => {
                movie_with_date.movie.title = title;
                Ok(())
            }
            None => {
                error!("Movie with id {} not found", id);
                Err(Error::NotFound(format!("Movie with id {} not found", id)))
            }
        }
    }

    fn change_movie_tags(&mut self, id: &MovieId, tags: Vec<String>) -> Result<(), Error> {
        info!("Changing tags of movie with id {}", id);

        let mut tags = tags;
        Self::process_tags(&mut tags);
        debug!("New tags: {:?}", tags);

        match self.movies.get_mut(id) {
            Some(movie_with_date) => {
                movie_with_date.movie.tags = tags;
                Ok(())
            }
            None => {
                error!("Movie with id {} not found", id);
                Err(Error::NotFound(format!("Movie with id {} not found", id)))
            }
        }
    }

    fn search_movies(&self, mut query: MovieSearchQuery) -> Result<Vec<MovieId>, Error> {
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
            match title_query {
                Some(ref title_query) => {
                    if !title_query.matches(&movie.title) {
                        continue;
                    }
                }
                None => {}
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

        movies.sort_unstable_by(|(_, lhs), (_, rhs)| lhs.cmp(&rhs));

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
            .map(|(id, movie)| (id.clone(), movie.date.clone()))
            .collect();

        movies.sort_unstable_by(|(_, lhs), (_, rhs)| lhs.cmp(&rhs));

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

    #[test]
    fn test_add_movie() {
        let mut index = SimpleMoviesIndex::new(&Options::default()).unwrap();

        let movies = create_test_movies();

        for movie in movies {
            let ret = index.add_movie(movie.clone());
            assert!(ret.is_ok());
        }
    }

    #[test]
    fn test_get_movie() {
        let mut index = SimpleMoviesIndex::new(&Options::default()).unwrap();
        let movies = create_test_movies();

        let movie_ids: Vec<MovieId> = movies
            .iter()
            .map(|m| index.add_movie(m.clone()).unwrap())
            .collect();

        for (id, movie) in movie_ids.iter().zip(movies.iter()) {
            let db_movie = index.get_movie(id).unwrap();

            assert_eq!(movie.title, db_movie.movie.title);
            assert_eq!(movie.description, db_movie.movie.description);

            let mut ref_tags = movie.tags.clone();
            SimpleMoviesIndex::process_tags(&mut ref_tags);
            assert_eq!(ref_tags, db_movie.movie.tags);
        }
    }

    #[test]
    fn test_remove_movie() {
        let mut index = SimpleMoviesIndex::new(&Options::default()).unwrap();
        let movies = create_test_movies();

        let mut movie_ids: Vec<MovieId> = movies
            .iter()
            .map(|m| index.add_movie(m.clone()).unwrap())
            .collect();

        movie_ids.sort();

        index.remove_movie(&movie_ids[0]).unwrap();
        assert!(index.remove_movie(&movie_ids[0]).is_err());

        let movie_ids = &movie_ids[1..];
        let mut listed_movie_ids = index.search_movies(Default::default()).unwrap();
        listed_movie_ids.sort();

        assert_eq!(movie_ids, listed_movie_ids);
    }

    #[test]
    fn test_query_movies() {
        let mut index = SimpleMoviesIndex::new(&Options::default()).unwrap();
        let movies = create_test_movies();

        movies.iter().for_each(|m| {
            let ret = index.add_movie(m.clone());
            assert!(ret.is_ok());
        });

        // test query 1A: Search all movies (ascending order by title)
        let mut query: MovieSearchQuery = Default::default();
        query.sorting_field = SortingField::Title;
        query.sorting_order = SortingOrder::Ascending;
        let movie_title: Vec<String> = index
            .search_movies(query)
            .unwrap()
            .iter()
            .map(|id| index.get_movie(id).unwrap().movie.title)
            .collect();
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
        let movie_title: Vec<String> = index
            .search_movies(query)
            .unwrap()
            .iter()
            .map(|id| index.get_movie(id).unwrap().movie.title)
            .collect();
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
        let search_result = index.search_movies(query).unwrap();
        let title_list = search_result
            .iter()
            .map(|id| index.get_movie(id).unwrap().movie.title.clone())
            .collect::<Vec<String>>();
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
        assert_eq!(index.search_movies(query).unwrap().len(), 0);
        let query = MovieSearchQuery {
            sorting_field: Default::default(),
            sorting_order: Default::default(),
            title: Some("*Boot".to_owned()),
            tags: vec![],
            start_index: None,
            num_results: None,
        };
        assert_eq!(
            index
                .search_movies(query)
                .unwrap()
                .iter()
                .map(|id| index.get_movie(id).unwrap().movie.title)
                .collect::<Vec<String>>(),
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
            index
                .search_movies(query)
                .unwrap()
                .iter()
                .map(|id| index.get_movie(id).unwrap().movie.title)
                .collect::<Vec<String>>(),
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
            index
                .search_movies(query)
                .unwrap()
                .iter()
                .map(|id| index.get_movie(id).unwrap().movie.title)
                .collect::<Vec<String>>(),
            ["Doctor Who", "E.T. the Extra-Terrestrial"]
        );
    }
}
