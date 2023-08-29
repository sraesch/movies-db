import { Box, Paper } from '@mui/material';
import * as React from 'react';
import VideoCard from './VideoCard';
import { service } from '../service/service';
import { MovieId, SortingField, SortingOrder } from '../service/types';
import YesNoDialog from './YesNoDialog';
import VideoPlayer from './VideoPlayer';
import VideoListFilter from './VideoListFilters';

export default function VideosList(): JSX.Element {
    const [movieIds, setMovieIds] = React.useState<string[]>([]);
    const [tagList, setTagList] = React.useState<[string, number][]>([]);
    const [deleteDialogOpen, setDeleteDialogOpen] = React.useState(false);
    const [movieToDelete, setMovieToDelete] = React.useState<MovieId | null>(null);
    const [movieToPlay, setMovieToPlay] = React.useState<MovieId | null>(null);

    // update list of movies
    const updateList = async () => {
        const movies = await service.searchMovies();
        const movieIds = movies.map((movie) => {
            return movie.id;
        });

        setTagList(await service.getTags());

        setMovieIds(movieIds);
    };

    // get list of movies
    React.useEffect(() => {
        updateList();

        service.registerVideoListUpdate(updateList);
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
    };

    const handleOnShow = (movieId: MovieId) => {
        console.log(`Showing movie ${movieId}`);
        setMovieToPlay(movieId);
    };

    const handleChangeTags = (tags: string[]) => {
        service.setSearchTags(tags);
    };

    const handleChangeSorting = (sorting_field: SortingField, sorting_order: SortingOrder) => {
        service.setSorting(sorting_field, sorting_order);
    };

    return (<Paper style={{
        display: 'flex',
        flexDirection: 'column',
        flexWrap: 'wrap',
        justifyContent: 'flex-start',
        alignItems: 'flex-start',
        flexGrow: 1,
    }}>
        <YesNoDialog
            title='Delete movie'
            msg='Are you sure you want to delete this movie?'
            open={deleteDialogOpen}
            onClose={() => setDeleteDialogOpen(false)}
            onAccept={() => handleOnDelete2()} />
        <VideoListFilter tagList={tagList} onChangeTags={handleChangeTags} onChangeSorting={handleChangeSorting} />
        <Box className='scrollButNoScrollbar' sx={{
            display: 'flex',
            flexDirection: 'row',
            flexWrap: 'wrap',
            justifyContent: 'flex-start',
            alignItems: 'flex-start',
            flexGrow: 1,
            overflowY: 'scroll',
            maxHeight: 'calc(100vh - 64px - 48px - 48px)',
        }}>
            {movieToPlay ? <VideoPlayer open={movieToPlay !== null} movieId={movieToPlay} onClose={() => setMovieToPlay(null)} /> : <></>}
            {
                movieIds.map((movieId) => {
                    return (<div key={movieId} style={{
                        margin: '16px',
                    }}>
                        <VideoCard movieId={movieId} onDelete={() => handleOnDelete1(movieId)} onShow={() => handleOnShow(movieId)} />
                    </div>);
                })
            }
        </Box>
    </Paper >)
}