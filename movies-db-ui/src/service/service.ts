import { Set } from "typescript";
import { MovieDetailed, MovieId, MovieSearchQuery, MovieSearchResult, MovieSubmit, SortingField, SortingOrder } from "./types";

export type ProgressUpdate = (progress: number, done: boolean) => void;

export type VideoListUpdate = () => void;

export class Service {
    private endpoint: string;
    private query: MovieSearchQuery;
    private videoListUpdate: Set<VideoListUpdate> = new Set<VideoListUpdate>();

    constructor(endpoint: string) {
        this.endpoint = endpoint;
        this.query = {
            sorting_field: SortingField.Date,
            sorting_order: SortingOrder.Descending,
            num_results: 50,
            start_index: 0,
        };
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

        this.notifyVideoListUpdate();

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
     * Returns a list of all tags sorted by the number of movies they are assigned to.
     *
     * @returns list of tags.
     */
    public async getTags(): Promise<[string, number][]> {
        const response = await fetch(`${this.endpoint}/movie/tags`);

        if (!response.ok) {
            throw new Error("Failed to get tags");
        }

        const tags = await response.json() as [string, number][];

        return tags;
    }

    /**
     * Deletes the movie with the given id.
     * 
     * @param id - The id of the movie to delete.
     **/
    public async removeMovie(id: MovieId): Promise<void> {
        const response = await fetch(`${this.endpoint}/movie?id=${id}`, {
            method: "DELETE",
        });

        if (!response.ok) {
            throw new Error("Failed to remove movie");
        }

        this.notifyVideoListUpdate();
    }

    /**
     * Returns a list of movies matching the given query in the given order.
     * 
     * @param query - The query to use for searching movies.
     * 
     * @returns a list of movie ids with title matching the given query in the given order.
     **/
    public async searchMovies(): Promise<MovieSearchResult> {
        const query = this.query;

        let queryString = `sorting_order=${query.sorting_order}&sorting_field=${query.sorting_field}`;
        if (query.title !== undefined) {
            queryString = queryString.concat(`&title=${query.title}`);
        }
        if (query.start_index !== undefined) {
            queryString = queryString.concat(`&start_index=${query.start_index}`);
        }
        if (query.num_results !== undefined) {
            queryString = queryString.concat(`&num_results=${query.num_results}`);
        }
        if (query.tags !== undefined) {
            query.tags.forEach((tag, index) => {
                queryString = queryString.concat(`&tags[${index}]=${tag}`);
            });
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
     * @returns the resource url for the preview of the video with the given id.
     */
    public getPreviewUrl(id: MovieId): string {
        return `${this.endpoint}/movie/screenshot?id=${id}`;
    }

    /**
     * Registers a callback to be called when the video list has been updated.
     * 
     * @param update - The callback to register.
     */
    public registerVideoListUpdate(update: VideoListUpdate): void {
        this.videoListUpdate.add(update);
    }

    /**
     * Sets the string to search for.
     * 
     * @param searchString - The string to search for.
     */
    public setSearchString(searchString: string): void {
        if (searchString.length === 0) {
            this.query.title = undefined;
            this.notifyVideoListUpdate();
            return;
        }

        if (searchString.startsWith("\"") && searchString.endsWith("\"")) {
            searchString = searchString.substring(1, searchString.length - 1);
            this.query.title = searchString;
        } else {
            if (!searchString.startsWith("*")) {
                searchString = "*" + searchString;
            }

            if (!searchString.endsWith("*")) {
                searchString = searchString + "*";
            }

            this.query.title = searchString;
        }

        this.notifyVideoListUpdate();
    }

    /**
     * Sets the tags to search for.
     * 
     * @param tags - The tags to search for.
     */
    public setSearchTags(tags: string[]): void {
        if (tags.length === 0) {
            this.query.tags = undefined;
        } else {
            this.query.tags = tags;
        }

        this.notifyVideoListUpdate();
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

    /**
     * Notifies all registered listeners that the video list has been updated.
     */
    private notifyVideoListUpdate(): void {
        this.videoListUpdate.forEach((update) => {
            update();
        });
    }
}

const SERVER_ADDRESS: string = (process.env.REACT_APP_SERVER_ADDRESS as string);
export const service = new Service(SERVER_ADDRESS);