import * as React from 'react';
import { Box, Chip } from '@mui/material';

export interface VideoListFilterProps {
    tagList: [string, number][];
    onChangeTags?: (tags: string[]) => void;
}

export default function VideoListFilter(props: VideoListFilterProps) {
    const [tags, setTags] = React.useState<string[]>([]);

    const handleToggleTag = (tagName: string) => {
        if (tagName === "") {
            setTags([]);
            return;
        }

        const index = tags.indexOf(tagName);
        if (index === -1) {
            setTags([...tags, tagName]);
        } else {
            setTags(tags.filter((_, i) => i !== index));
        }
    };

    React.useEffect(() => {
        if (props.onChangeTags) {
            props.onChangeTags(tags);
        }
    }, [tags, props.onChangeTags]);

    return (
        <Box sx={{
            overflow: 'hidden',
            maxWidth: '100%',
            display: 'flex',
            flexDirection: 'row',
            justifyContent: 'center',
        }}>
            <Box className="scrollButNoScrollbar" sx={{
                display: 'flex',
                flexDirection: 'row',
                flexWrap: 'nowrap',
                padding: '10px',
                maxWidth: '90%',
                overflowX: 'scroll',
            }}>
                <Chip onClick={() => handleToggleTag("")} variant={tags.length === 0 ? 'filled' : 'outlined'} label="All" clickable />
                {props.tagList.map(([tagName, _]) => {
                    return (<Box key={tagName} sx={{ marginLeft: '8px' }}>
                        <Chip onClick={() => handleToggleTag(tagName)} variant={tags.indexOf(tagName) === -1 ? 'outlined' : 'filled'} label={tagName} clickable />
                    </Box>);
                })}
            </Box>
        </Box>
    );
}