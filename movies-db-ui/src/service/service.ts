import { MovieId, MovieSubmit } from "./types";

export class Service {
    private endpoint: string;

    constructor(endpoint: string) {
        this.endpoint = endpoint;
    }

    /**
     * Submits the infos for a new movie, but without the respective file.
     * 
     * @param movie - The movie to submit.
     *
     * @returns the id of the submitted movie.
     */
    public async submitMovie(movie: MovieSubmit): Promise<MovieId> {
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

        const { id } = await response.json();
        return id;
    }
}

export const service = new Service("http://localhost:3030/api/v1");