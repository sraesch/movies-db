use uuid::Uuid;

pub type MovieId = String;

/// Generates and returns a new movie random ID
pub fn generate_movie_id() -> MovieId {
    Uuid::new_v4().to_string()
}
