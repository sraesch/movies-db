import * as React from 'react';
import InputBase from '@mui/material/InputBase';
import SearchIcon from '@mui/icons-material/Search';
import { IconButton, MenuItem, Paper } from '@mui/material';

export interface SearchProps {
    onSearch?: (query: string) => void;
}

export default function Search(props: SearchProps) {
    const [query, setQuery] = React.useState("");

    const handleSearch = () => {
        if (props.onSearch) {
            props.onSearch(query);
        }
    };

    return (
        <Paper
            sx={{ p: '2px 4px', flexGrow: 1, display: 'flex', alignItems: 'center' }}
        >
            <IconButton sx={{ p: '10px' }} aria-label="menu">
                <MenuItem />
            </IconButton>
            <InputBase
                value={query}
                sx={{ ml: 1, flex: 1 }}
                placeholder="Search"
                inputProps={{ 'aria-label': 'search' }}
                onChange={e => setQuery(e.target.value)}
                onKeyDown={e => {
                    if (e.key === 'Enter') {
                        handleSearch();
                    }
                }}
            />
            <IconButton type="button" sx={{ p: '10px' }} onClick={() => handleSearch()}>
                <SearchIcon />
            </IconButton>
        </Paper>
    );
}