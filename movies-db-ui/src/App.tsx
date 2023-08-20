import * as React from 'react';
import './App.css';
import AddVideoDialog from './components/AddVideoDialog';
import { Button } from '@mui/material';
import { MovieSubmit } from './service/types';
import { service } from './service/service';

function App() {
  const [open, setOpen] = React.useState(false);

  const handleAddVideo = async (info: MovieSubmit, file: File): Promise<void> => {
    console.log(`Submit Video: ${JSON.stringify(info)}`);
    await service.submitMovie(info, file);
  };

  return (
    <div className="App">
      <main>
        <Button variant='contained' onClick={() => setOpen(true)}>Add Video</Button>
        <AddVideoDialog open={open} onClose={() => setOpen(false)} onSubmit={handleAddVideo} />
      </main>
    </div>
  );
}

export default App;
