import * as React from 'react';
import Card from '@mui/material/Card';
import CardHeader from '@mui/material/CardHeader';
import CardMedia from '@mui/material/CardMedia';
import CardContent from '@mui/material/CardContent';
import CardActions from '@mui/material/CardActions';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import ClearIcon from '@mui/icons-material/Clear';
import { MovieDetailed, MovieId } from '../service/types';
import { CardActionArea, Chip } from '@mui/material';
import { service } from '../service/service';

import NoVideo from '../img/no_video.png';


export interface VideoCardProps {
    movieId: MovieId;
    onDelete?: () => void;
    onShow?: () => void;
}

export default function VideoCard(props: VideoCardProps): JSX.Element {
    const [movieInfo, setMovieInfo] = React.useState<MovieDetailed | null>(null);
    const [movieURL, setMovieURL] = React.useState<string | null>(null);
    // try to load the movie info
    React.useEffect(() => {
        service.getMovie(props.movieId).then((movie) => {
            setMovieInfo(movie);

            if (movie.movie_file_info) {
                setMovieURL(service.getMovieUrl(props.movieId));
            }
        });
    }, [props.movieId]);


    if (!movieInfo) {
        return <div></div>
    }

    const handleOnDelete = () => {
        if (props.onDelete) {
            props.onDelete();
        }
    };

    const movieDate = new Date(movieInfo.date);

    const { movie } = movieInfo;
    const description = movie.description ? (movie.description.length > 100 ? movie.description.substring(0, 100) + '...' : movie.description) : '';

    const handleOnShow = () => {
        if (props.onShow) {
            props.onShow();
        }
    };

    return (
        <Card sx={{ maxWidth: 345 }}>
            <CardHeader
                action={
                    <IconButton aria-label="settings" onClick={handleOnDelete}>
                        <ClearIcon />
                    </IconButton>
                }
                title={movie.title}
                subheader={movieDate.toLocaleDateString() + ' ' + movieDate.toLocaleTimeString()}
            />
            <CardActionArea onClick={handleOnShow}>
                {movieURL ? <CardMedia
                    component="video"
                    height="194"
                    src={movieURL}
                /> : <CardMedia
                    component="img"
                    height="194"
                    image={NoVideo}
                    alt="Video not found"
                />}
                <CardContent>
                    <Typography variant="body2" color="text.secondary">
                        {description}
                    </Typography>
                </CardContent>
            </CardActionArea>
            <CardActions disableSpacing>
                {movie.tags ? movie.tags.map((tag, index) => {
                    return <Chip key={index} label={tag} />
                }) : <div></div>}
            </CardActions>

        </Card>
    );
}