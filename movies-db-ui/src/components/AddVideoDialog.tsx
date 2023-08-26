import { Button, Chip, Dialog, DialogActions, DialogContent, DialogTitle, Fab, TextField, Typography } from '@mui/material';
import Add from '@mui/icons-material/Add';
import * as React from 'react';
import { MovieSubmit } from '../service/types';

export interface AddVideoDialogProps {
    open: boolean;
    onClose: () => void;
    onSubmit?: (info: MovieSubmit, file: File) => Promise<void>;
}

export default function AddVideoDialog(props: AddVideoDialogProps): JSX.Element {
    const [title, setTitle] = React.useState<string>('');
    const [description, setDescription] = React.useState<string>('');
    const [tags, setTags] = React.useState<string[]>([]);
    const [tag, setTag] = React.useState<string>('');
    const [file, setFile] = React.useState<File | null>(null);

    // reset the form when the dialog is re-opened
    React.useEffect(() => {
        if (props.open) {
            setFile(null);
            setDescription('');
            setTitle('');
            setTags([]);
        }
    }, [props.open]);

    const handleDeleteTag = (tag: string) => {
        setTags(tags.filter(t => t !== tag));
    }

    const handleAddTag = (tag: string) => {
        // make sure the tag is not already in the list
        if (tags.find(t => t === tag)) {
            return;
        } else if (tag.length > 0) {
            setTags([...tags, tag]);
            setTag('');
        }
    }

    const onFilechange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files) {
            setFile(e.target.files[0]);
        }
    }

    const createMovieSubmitInfo = (): MovieSubmit | null => {
        if (title.length === 0) {
            return null;
        }

        return {
            title,
            description,
            tags
        };
    }

    const handleSubmit = () => {
        if (!props.onSubmit) {
            props.onClose();
            return;
        }

        const info = createMovieSubmitInfo();
        if (info && file && props.onSubmit) {
            props.onClose();
            props.onSubmit(info, file);
        }
    }

    return (
        <Dialog open={props.open}
            color='inherit'
            onClose={props.onClose}
            aria-label='close'>
            <DialogTitle>Add Video</DialogTitle>
            <DialogContent>
                <TextField
                    error={title.length === 0}
                    autoFocus
                    margin="dense"
                    id="title"
                    label="Title"
                    type="text"
                    fullWidth
                    variant="standard"
                    value={title}
                    onChange={(e) => setTitle(e.target.value)}
                />
                <TextField
                    autoFocus
                    multiline
                    rows={8}
                    margin="dense"
                    id="title"
                    label="Description"
                    type="text"
                    fullWidth
                    variant="standard"
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                />
                <div style={{
                    display: 'flex',
                    flexDirection: 'row',
                    justifyContent: 'flex-start',
                    flexWrap: 'wrap',
                    listStyle: 'none',
                    height: '64px',
                    padding: '8px',
                    overflowY: 'auto',
                    overflowX: 'clip'
                }}>
                    {tags.map(tag => {
                        return (
                            <div key={tag}>
                                <Chip
                                    label={tag}
                                    onDelete={() => handleDeleteTag(tag)}
                                />
                            </div>
                        );
                    })}
                </div>
                <div style={{ display: "flex", flexDirection: "row", flexWrap: 'nowrap', justifyContent: 'space-between', alignItems: 'center' }}>
                    <TextField
                        autoFocus
                        margin="dense"
                        id="title"
                        style={{ flexGrow: 1, marginRight: '16px' }}
                        label="Add Tag"
                        type="text"
                        variant="standard"
                        value={tag}
                        onChange={(e) => setTag(e.target.value)}
                    />
                    <Fab color='secondary' size='small' disabled={tag === ''} onClick={() => handleAddTag(tag)}><Add /></Fab>
                </div>
                <div style={{ marginTop: "32px", flex: 'row' }}>
                    <input
                        accept='video/*'
                        style={{ display: 'none' }}
                        onChange={onFilechange}
                        id="raised-button-file"
                        type="file"
                    />
                    <label htmlFor="raised-button-file">
                        <Button variant='contained' color='secondary' component="span">
                            Choose Video
                        </Button>
                    </label>
                    <Typography maxWidth={'128px'} variant='body1' style={{ marginTop: '16px', flexGrow: 0 }}>
                        {file ? file.name : 'No file selected'}
                    </Typography>
                </div>
            </DialogContent>
            <DialogActions>
                <Button color='secondary' onClick={props.onClose}>Cancel</Button>
                <Button color='primary' disabled={title.length === 0 || !file} onClick={handleSubmit} autoFocus>
                    Add
                </Button>
            </DialogActions>
        </Dialog >
    );
} 