use actix_web::http::header;
use actix_web::HttpResponse;
use actix_web::{web, Responder, Result};
use futures::{StreamExt, TryStreamExt};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

use tokio_util::io::ReaderStream;

use crate::{Error, Options};

pub struct ServiceHandler {
    options: Options,
}

impl ServiceHandler
{
    /// Creates a new instance of the service handler.
    ///
    /// # Arguments
    /// * `options` - The options for the service handler.
    pub fn new(options: Options) -> Result<Self, Error> {
        Ok(Self { options })
    }

    /// Handles the request to create a preview image.
    ///
    /// # Arguments
    /// * `id` - The id of the movie to create a preview image for.
    pub async fn handle_create_preview(&self, id: &str) -> Result<impl Responder> {
        Ok(actix_web::HttpResponse::Ok())
    }
}
