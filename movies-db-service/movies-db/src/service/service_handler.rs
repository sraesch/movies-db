use crate::{Error, MovieStorage, MoviesIndex, Options};

pub struct ServiceHandler<I, S>
where
    I: MoviesIndex,
    S: MovieStorage,
{
    index: I,
    storage: S,
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
}
