import * as React from 'react';
import { MovieDetailed, MovieId } from '../service/types';
import { Chip, Dialog, Paper, Typography } from '@mui/material';
import { service } from '../service/service';

export interface VideoPlayerProps {
    movieId: MovieId;

    open: boolean;
    onClose: () => void;
}

export default function VideoPlayer(props: VideoPlayerProps) {
    const [movieURL, setMovieURL] = React.useState<string | null>(null);
    const [movieInfo, setMovieInfo] = React.useState<MovieDetailed | null>(null);

    React.useEffect(() => {
        setMovieURL(service.getMovieUrl(props.movieId));

        service.getMovie(props.movieId).then((movie) => {
            setMovieInfo(movie);
        });

    }, [props.movieId]);

    if (!movieInfo || !movieURL) {
        return <div></div>
    }

    const movie = movieInfo.movie;

    return (
        <Dialog
            open={props.open}
            onClose={props.onClose}
            aria-labelledby="alert-dialog-title"
            aria-describedby="alert-dialog-description"
            maxWidth="xl"
            fullWidth
        >
            <Paper sx={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                padding: '16px',
            }}>
                <Typography variant="h4">
                    {movie.title}
                </Typography>
                <Typography variant="h6" color="text.secondary" style={{ marginTop: '16px' }}>
                    {movieInfo.date ? new Date(movieInfo.date).toLocaleString() : ''}
                </Typography>
                <div style={{
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                }}>
                    <video controls src={movieURL} style={{
                        marginTop: '16px',
                        height: '50vh',
                    }} />
                    <div style={{
                        marginTop: '16px',
                        display: 'flex',
                        flexDirection: 'row',
                        alignItems: 'flex-start',
                        flexGrow: 1,
                    }}>
                        {movie.tags ? movie.tags.map((tag, index) => {
                            return <Chip key={index} label={tag} style={{ marginLeft: '8px' }} />
                        }) : <div></div>}
                    </div>
                </div>
                <Typography variant="body2" color="text.secondary" style={{ marginTop: '16px' }}>
                    {movie.description ? movie.description : ''}
                </Typography>
            </Paper>
        </Dialog>
    );
}