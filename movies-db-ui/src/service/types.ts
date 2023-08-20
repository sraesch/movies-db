export type MovieId = string;

/**
 * The interface for submitting the info of a movie.
 */
export interface MovieSubmit {
    title: string;
    description?: string;
    tags?: string[];
}