import * as React from 'react';
import './App.css';
import AddVideoDialog from './components/AddVideoDialog';
import { AppBar, Box, Button, Toolbar } from '@mui/material';
import { MovieSubmit } from './service/types';
import { service } from './service/service';
import LoadingDialog, { LoadingDialogProps } from './components/LoadingDialog';
import VideosList from './components/VideosList';
import { createTheme, ThemeProvider } from '@mui/material/styles';

const theme = createTheme({
  palette: {
    mode: 'dark',
    primary: {
      main: '#7e57c2',
    },

    secondary: {
      main: '#3f51b5',
    },
  },
});

function App() {
  const [videoDialogOpen, setVideoDialogOpen] = React.useState(false);
  const [loadingDialogProps, setLoadingDialogProps] = React.useState<LoadingDialogProps>({ open: false, title: "", msg: "", progress: 0 });

  const handleAddVideo = async (info: MovieSubmit, file: File): Promise<void> => {
    console.log(`Submit Video: ${JSON.stringify(info)}`);
    await service.submitMovie(info, file, (progress: number, done: boolean) => {
      setLoadingDialogProps({ open: !done, title: "Uploading Video", msg: "Sending data...", progress: progress });
    });
    setLoadingDialogProps({ open: false, title: "", msg: "", progress: 0 });
  };

  return (
    <ThemeProvider theme={theme}>
      <div className="App">
        <header>
          <LoadingDialog open={loadingDialogProps.open} title={loadingDialogProps.title} msg={loadingDialogProps.msg} progress={loadingDialogProps.progress} />
          <AddVideoDialog open={videoDialogOpen} onClose={() => setVideoDialogOpen(false)} onSubmit={handleAddVideo} />
        </header>
        <main>
          <div>
            <Box sx={{ flexGrow: 1 }}>
              <AppBar position="static">
                <Toolbar>
                  <Button color="primary" onClick={() => setVideoDialogOpen(true)}>Add Video</Button>
                </Toolbar>
              </AppBar>
            </Box>
          </div>
          <VideosList />
        </main>
      </div>
    </ThemeProvider>
  );
}

export default App;
