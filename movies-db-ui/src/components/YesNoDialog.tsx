import * as React from 'react';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import DialogContentText from '@mui/material/DialogContentText';
import DialogTitle from '@mui/material/DialogTitle';

export interface YesNoDialogProps {
    title: string;
    msg: string;

    open: boolean;

    onClose: () => void;
    onAccept: () => void;
}

export default function YesNoDialog(props: YesNoDialogProps) {
    const handleOnAccept = () => {
        props.onAccept();
        props.onClose();
    };

    return (
        <Dialog
            open={props.open}
            onClose={props.onClose}
            aria-labelledby="alert-dialog-title"
            aria-describedby="alert-dialog-description"
        >
            <DialogTitle id="alert-dialog-title">
                {props.title}
            </DialogTitle>
            <DialogContent>
                <DialogContentText id="alert-dialog-description">
                    {props.msg}
                </DialogContentText>
            </DialogContent>
            <DialogActions>
                <Button onClick={props.onClose}>Disagree</Button>
                <Button onClick={handleOnAccept} autoFocus>
                    Accept
                </Button>
            </DialogActions>
        </Dialog>
    );
}