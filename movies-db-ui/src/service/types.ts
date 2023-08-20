export type MovieId = string;

/**
 * The interface for submitting the info of a movie.
 */
export interface MovieSubmit {
    title: string;
    description?: string;
    tags?: string[];
}

export interface MovieFileInfo {
    /// the extension of the movie file in lower case, e.g., "mp4"
    extension: string,

    // the mime type of the movie file, e.g., "video/mp4"
    mime_type: string,
}

/**
 * A detailed movie with additional information.
 */
export interface MovieDetailed {
    movie: MovieSubmit;
    movie_file_info?: MovieFileInfo;
    date: string;
}