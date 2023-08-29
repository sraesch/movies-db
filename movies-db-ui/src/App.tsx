import * as React from 'react';
import './App.css';
import AddVideoDialog from './components/AddVideoDialog';
import { AppBar, Box, Button, Toolbar, Typography } from '@mui/material';
import { MovieSubmit } from './service/types';
import { service } from './service/service';
import LoadingDialog, { LoadingDialogProps } from './components/LoadingDialog';
import VideosList from './components/VideosList';
import { createTheme, ThemeProvider } from '@mui/material/styles';
import AddIcon from '@mui/icons-material/Add';
import Search from './components/Search';

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
                <Toolbar sx={{ display: 'flex', flexDirection: 'row', justifyContent: 'space-between' }}>
                  <Box sx={{ flexGrow: 1 }}>
                    <Typography
                      variant="h6"
                      noWrap
                      component="a"
                      href="/"
                      sx={{
                        mr: 2,
                        display: { xs: 'none', md: 'flex' },
                        fontFamily: 'monospace',
                        fontWeight: 700,
                        letterSpacing: '.2rem',
                        color: 'inherit',
                        textDecoration: 'none',
                      }}
                    >
                      Movie DB
                    </Typography>
                  </Box>
                  <Box sx={{ flexGrow: 2 }}>
                    <Search onSearch={s => service.setSearchString(s)} />
                  </Box>
                  <Box sx={{ flexGrow: 1, display: 'flex', justifyContent: 'flex-end' }}>
                    <Button color="primary" variant='contained' onClick={() => setVideoDialogOpen(true)} startIcon={<AddIcon />}>
                      Add Video
                    </Button>
                  </Box>
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
