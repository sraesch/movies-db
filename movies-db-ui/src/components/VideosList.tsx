import { Paper } from '@mui/material';
import * as React from 'react';
import VideoCard from './VideoCard';
import { service } from '../service/service';
import { MovieDetailed, MovieId, MovieSearchQuery, SortingField, SortingOrder } from '../service/types';

export default function VideosList(): JSX.Element {
    const [movieIds, setMovieIds] = React.useState<string[]>([]);


    // get list of movies
    React.useEffect(() => {
        const query: MovieSearchQuery = {
            sorting_field: SortingField.Date,
            sorting_order: SortingOrder.Descending,
        }

        service.searchMovies(query).then((movies) => {
            const movieIds = movies.map((movie) => {
                return movie.id;
            });

            setMovieIds(movieIds);
        });
    }, []);


    return (<Paper style={{
        display: 'flex',
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'flex-start',
        alignItems: 'flex-start',
    }}>
        {movieIds.map((movieId, index) => {
            return (<div style={{
                margin: '16px',
            }}>
                <VideoCard key={index} movieId={movieId} />
            </div>);
        })}

    </Paper>)
}