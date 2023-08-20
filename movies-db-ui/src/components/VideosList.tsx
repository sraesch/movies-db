import { Paper } from '@mui/material';
import * as React from 'react';
import VideoCard from './VideoCard';
import { service } from '../service/service';
import { MovieId, MovieSearchQuery, SortingField, SortingOrder } from '../service/types';
import YesNoDialog from './YesNoDialog';

export default function VideosList(): JSX.Element {
    const [movieIds, setMovieIds] = React.useState<string[]>([]);
    const [deleteDialogOpen, setDeleteDialogOpen] = React.useState(false);
    const [movieToDelete, setMovieToDelete] = React.useState<MovieId | null>(null);

    // update list of movies
    const updateList = async () => {
        const query: MovieSearchQuery = {
            sorting_field: SortingField.Date,
            sorting_order: SortingOrder.Descending,
        }

        const movies = await service.searchMovies(query);
        const movieIds = movies.map((movie) => {
            return movie.id;
        });

        setMovieIds(movieIds);
    };

    // get list of movies
    React.useEffect(() => {
        updateList();
    }, []);

    const handleOnDelete1 = (movieId: MovieId) => {
        console.log(`Deleting movie ${movieId}`);
        setMovieToDelete(movieId);
        setDeleteDialogOpen(true);
    };

    const handleOnDelete2 = async () => {
        if (movieToDelete === null) {
            return;
        }

        await service.removeMovie(movieToDelete);
        await updateList();
    };

    return (<Paper style={{
        display: 'flex',
        flexDirection: 'row',
        flexWrap: 'wrap',
        justifyContent: 'flex-start',
        alignItems: 'flex-start',
    }}>
        <YesNoDialog
            title='Delete movie'
            msg='Are you sure you want to delete this movie?'
            open={deleteDialogOpen}
            onClose={() => setDeleteDialogOpen(false)}
            onAccept={() => handleOnDelete2()} />
        {movieIds.map((movieId) => {
            return (<div key={movieId} style={{
                margin: '16px',
            }}>
                <VideoCard movieId={movieId} onDelete={() => handleOnDelete1(movieId)} />
            </div>);
        })}

    </Paper>)
}