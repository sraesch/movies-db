import { Box, CircularProgress, Paper, Typography } from "@mui/material";
import React from "react";

export interface LoadingDialogProps {
    title: string;
    progress?: number;
    msg?: string;
    open: boolean;
}

export default function LoadingDialog(props: LoadingDialogProps): JSX.Element {
    const progress = props.progress ? Math.round(props.progress) : 0;

    return (
        !props.open ? <div></div> :
            <div style={{
                position: 'fixed',
                zIndex: 40,
                left: 0,
                top: 0,
                width: '100%',
                height: '100%',
                overflow: 'auto',
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
                backgroundColor: 'rgba(14, 20, 24, 0.8)',
            }}>
                <Paper style={{
                    width: '512px',
                }}>
                    <Typography variant='h6' style={{
                        textAlign: 'center',
                        paddingTop: '32px',
                    }}>
                        {props.title}
                    </Typography>
                    <div style={{ display: 'flex', flexDirection: 'row', alignItems: "center", justifyContent: "center", margin: '32px' }}>
                        <Typography variant='body1' style={{
                            textAlign: 'center',
                            paddingRight: '16px',
                        }}>
                            {props.msg ? props.msg : ""}
                        </Typography>
                        <Box sx={{ position: 'relative', display: 'inline-flex' }}>
                            <CircularProgress variant="determinate" value={progress} />
                            <Box
                                sx={{
                                    top: 0,
                                    left: 0,
                                    bottom: 0,
                                    right: 0,
                                    position: 'absolute',
                                    display: 'flex',
                                    alignItems: 'center',
                                    justifyContent: 'center',
                                }}
                            >
                                <Typography
                                    variant="caption"
                                    component="div"
                                    color="text.secondary"
                                >{`${progress}%`}</Typography>
                            </Box>
                        </Box>
                    </div>
                </Paper>

            </div >
    );
}
