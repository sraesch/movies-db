import { MovieId, MovieSubmit } from "./types";

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
     *
     * @returns the id of the submitted movie.
     */
    public async submitMovie(movie: MovieSubmit, file: File): Promise<MovieId> {
        const id = await this.submitMovieInfo(movie);
        await this.submitMovieFile(id, file);

        return id;
    }

    /**
     * Submits a video file for the given movie id.
     * 
     * @param id - The id of the movie to submit.
     * @param file - The file to submit.
     */
    private async submitMovieFile(id: MovieId, file: File): Promise<void> {
        const formData = new FormData();
        formData.append("video", file);
        const response = await fetch(`${this.endpoint}/movie/file?id=${id}`, { method: "POST", body: formData });

        if (!response.ok) {
            throw new Error("Failed to submit movie");
        }
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