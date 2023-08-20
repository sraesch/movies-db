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

/**
 * The sorting order for the movies.
 **/
export enum SortingField {
    Title = "title",
    Date = "date",
}

/**
 * The sorting order for the movies.
 **/
export enum SortingOrder {
    Ascending = "ascending",
    Descending = "descending",
}

/**
 * A query for searching movies in the database.
 **/
export interface MovieSearchQuery {
    /// The field used for sorting
    sorting_field: SortingField,

    /// The order used for sorting
    sorting_order: SortingOrder,

    /// Optionally, a search string for the title of the movie. If provided, only movies whose
    /// title matches the search string will be returned.
    /// Wildcards are supported, e.g., *foo* will match any movie whose title contains "foo".
    title?: string,

    /// A sorted list of lower case tags that must match the movie.
    tags?: string[],

    /// Optionally, the start index of the movies to return.
    start_index?: number,

    /// Optionally, the maximal number of results to return.
    num_results?: number,
}

export interface MovieSearchResultEntry {
    id: MovieId;
    title: string;
}

export type MovieSearchResult = MovieSearchResultEntry[];