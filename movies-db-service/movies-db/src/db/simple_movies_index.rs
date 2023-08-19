use std::collections::HashMap;

use log::{debug, error, info};
use wildmatch::WildMatch;

use crate::{
    generate_movie_id, Error, Movie, MovieId, MovieSearchQuery, MovieWithDate, MoviesIndex, Options,
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
            date: chrono::Utc::now().to_rfc3339(),
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

    fn list_movies(&self) -> Vec<MovieId> {
        info!("Listing movies");

        self.movies.keys().cloned().collect()
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

        // create wildcard query if provided
        let title_query: Option<WildMatch> = query.title.map(|s| WildMatch::new(&s));

        let mut movie_ids = Vec::new();
        for (id, movie_with_date) in self.movies.iter() {
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

            movie_ids.push(id.clone());
        }

        Ok(movie_ids)
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
    fn test_list_movies() {
        let mut index = SimpleMoviesIndex::new(&Options::default()).unwrap();
        let movies = create_test_movies();

        let movie_ids: Vec<MovieId> = movies
            .iter()
            .map(|m| index.add_movie(m.clone()).unwrap())
            .collect();

        let mut sorted_movie_ids = movie_ids.clone();
        sorted_movie_ids.sort();

        assert_eq!(movies.len(), movie_ids.len());

        let mut listed_movie_ids = index.list_movies();
        listed_movie_ids.sort();
        assert_eq!(sorted_movie_ids, listed_movie_ids);
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
        let mut listed_movie_ids = index.list_movies();
        listed_movie_ids.sort();

        assert_eq!(movie_ids, listed_movie_ids);
    }

    #[test]
    fn test_query_movies() {
        let mut index = SimpleMoviesIndex::new(&Options::default()).unwrap();
        let movies = create_test_movies();

        let movie_ids: Vec<MovieId> = movies
            .iter()
            .map(|m| index.add_movie(m.clone()).unwrap())
            .collect();

        // test query 1: Search all movies
        let query = MovieSearchQuery {
            title: None,
            tags: vec![],
        };
        assert_eq!(index.search_movies(query).unwrap().len(), movie_ids.len());

        // test query 2: Search only science fiction movies
        let query = MovieSearchQuery {
            title: None,
            tags: vec!["Sci-Fi".to_owned()],
        };
        let search_result = index.search_movies(query).unwrap();
        let mut title_list = search_result
            .iter()
            .map(|id| index.get_movie(id).unwrap().movie.title.clone())
            .collect::<Vec<String>>();
        title_list.sort();
        assert_eq!(
            title_list,
            vec![
                "Doctor Who".to_owned(),
                "E.T. the Extra-Terrestrial".to_owned()
            ]
        );

        // test query 3: Search 'Das Boot'
        let query = MovieSearchQuery {
            title: Some("Boot".to_owned()),
            tags: vec![],
        };
        assert_eq!(index.search_movies(query).unwrap().len(), 0);
        let query = MovieSearchQuery {
            title: Some("*Boot".to_owned()),
            tags: vec![],
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
    }
}
