import { MovieDetailed, MovieId, MovieSearchQuery, MovieSearchResult, MovieSubmit } from "./types";

export type ProgressUpdate = (progress: number, done: boolean) => void;

export class Service {
    private endpoint: string;

    constructor(endpoint: string) {
        this.endpoint = endpoint;
    }

    /**
     * Submits a video with additional information.
     * 
     * @param movie - The movie to submit.
     * @param file - The file to submit.
     * @param progressUpdate - An optional callback to receive progress updates.
     *
     * @returns the id of the submitted movie.
     */
    public async submitMovie(movie: MovieSubmit, file: File, progressUpdate?: ProgressUpdate): Promise<MovieId> {
        const id = await this.submitMovieInfo(movie);
        await this.submitMovieFile(id, file, progressUpdate);

        return id;
    }

    /**
     * Returns detailed information for the given movie id.
     * 
     * @param id - The id of the movie to get.
     *
     * @returns detailed information for the given movie id.
     */
    public async getMovie(id: MovieId): Promise<MovieDetailed> {
        const response = await fetch(`${this.endpoint}/movie?id=${id}`);

        if (!response.ok) {
            throw new Error("Failed to get movie");
        }

        const movie = await response.json() as MovieDetailed;

        return movie;
    }

    /**
     * Returns a list of movies matching the given query in the given order.
     * 
     * @param query - The query to use for searching movies.
     * 
     * @returns a list of movie ids with title matching the given query in the given order.
     **/
    public async searchMovies(query: MovieSearchQuery): Promise<MovieSearchResult> {
        const queryString = `sorting_order=${query.sorting_order}&sorting_field=${query.sorting_field}`;
        if (query.title) {
            queryString.concat(`&title=${query.title}`);
        }
        if (query.start_index) {
            queryString.concat(`&start_index=${query.start_index}`);
        }
        if (query.num_results) {
            queryString.concat(`&num_results=${query.num_results}`);
        }

        const response = await fetch(`${this.endpoint}/movie/search?${queryString}`, {
            method: "GET",
        });

        if (!response.ok) {
            throw new Error("Failed to search movies");
        }

        const movies = await response.json() as MovieSearchResult;

        return movies;
    }

    /**
     * @returns the resource url for the movie file with the given id.
     */
    public getMovieUrl(id: MovieId): string {
        return `${this.endpoint}/movie/file?id=${id}`;
    }

    /**
     * Submits a video file for the given movie id.
     * 
     * @param id - The id of the movie to submit.
     * @param file - The file to submit.
     * @param progressUpdate - An optional callback to receive progress updates.
     */
    private submitMovieFile(id: MovieId, file: File, progressUpdate?: ProgressUpdate): Promise<void> {
        const handleProgressUpdate = (progress: number, done: boolean) => {
            if (progressUpdate) {
                progressUpdate(progress, done);
            }
        }

        handleProgressUpdate(0, false);
        const formData = new FormData();
        formData.append("video", file);

        const request = new XMLHttpRequest();
        request.open('POST', `${this.endpoint}/movie/file?id=${id}`);

        const promise = new Promise<void>((resolve, reject) => {
            // upload progress event
            request.upload.addEventListener('progress', (e) => {
                // upload progress as percentage
                let percent_completed = (e.loaded / e.total) * 100;
                handleProgressUpdate(percent_completed, false);
            });

            // request finished event
            request.addEventListener('load', (e) => {
                handleProgressUpdate(100, true);
                const isOk = request.status === 200 || request.status === 201 || request.status === 202;
                if (isOk) {
                    resolve();
                } else {
                    reject(request.response);
                }
            });
        });

        // send POST request to server
        request.send(formData);

        return promise;
    }

    /**
     * Submits video info.
     * 
     * @param movie - The movie info to submit.
     * 
     * @returns the id of the submitted movie.
     */
    private async submitMovieInfo(movie: MovieSubmit): Promise<MovieId> {
        const response = await fetch(`${this.endpoint}/movie`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(movie),
        });

        if (!response.ok) {
            throw new Error("Failed to submit movie");
        }

        const id = await response.text();

        return id;
    }
}

export const service = new Service("http://localhost:3030/api/v1");