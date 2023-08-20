import { Paper } from '@mui/material';
import * as React from 'react';
import VideoCard from './VideoCard';
import { service } from '../service/service';

export default function VideosList(): JSX.Element {
    const [movieIds, setMovieIds] = React.useState<string[]>([]);

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