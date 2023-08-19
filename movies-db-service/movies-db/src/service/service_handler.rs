use crate::{MovieStorage, MoviesIndex, Options};

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
    pub fn new(options: Options) -> Self {
        let index = I::new(&options).unwrap();
        let storage = S::new(&options).unwrap();

        Self { index, storage }
    }
}
