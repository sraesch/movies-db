import * as React from 'react';
import { Box, Chip, IconButton, Menu, MenuItem } from '@mui/material';
import IconSort from '@mui/icons-material/SortRounded';
import ListItemIcon from '@mui/material/ListItemIcon';
import Check from '@mui/icons-material/Check';
import { SortingField, SortingOrder } from '../service/types';

enum SortingOption {
    TitleAscending,
    TitleDescending,
    DateAscending,
    DateDescending,
}

export interface VideoListFilterProps {
    tagList: [string, number][];
    onChangeTags?: (tags: string[]) => void;
    onChangeSorting?: (sorting_field: SortingField, sorting_order: SortingOrder) => void;
}

const menuOptions = [
    {
        label: "Date descending",
        value: SortingOption.DateDescending,
    },
    {
        label: "Date ascending",
        value: SortingOption.DateAscending,
    },
    {
        label: "Title descending",
        value: SortingOption.TitleDescending,
    },
    {
        label: "Title ascending",
        value: SortingOption.TitleAscending,
    },
]

export default function VideoListFilter(props: VideoListFilterProps) {
    const [tags, setTags] = React.useState<string[]>([]);
    const [anchorEl, setAnchorEl] = React.useState<null | HTMLElement>(null);
    const [sortingOption, setSortingOption] = React.useState<SortingOption>(SortingOption.DateDescending);

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
    }, [tags, props]);

    React.useEffect(() => {
        const handleSorting = (option: SortingOption) => {
            if (!props.onChangeSorting) {
                return;
            }

            switch (option) {
                case SortingOption.DateAscending:
                    props.onChangeSorting(SortingField.Date, SortingOrder.Ascending);
                    break;
                case SortingOption.DateDescending:
                    props.onChangeSorting(SortingField.Date, SortingOrder.Descending);
                    break;
                case SortingOption.TitleAscending:
                    props.onChangeSorting(SortingField.Title, SortingOrder.Ascending);
                    break;
                case SortingOption.TitleDescending:
                    props.onChangeSorting(SortingField.Title, SortingOrder.Descending);
                    break;
            }
        };

        handleSorting(sortingOption);
    }, [sortingOption, props]);

    const handleMenu = (event: React.MouseEvent<HTMLElement>) => {
        setAnchorEl(event.currentTarget);
    };

    const handleClose = () => {
        setAnchorEl(null);
    };

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
                width: '80%',
                overflowX: 'scroll',
            }}>
                <Chip onClick={() => handleToggleTag("")} variant={tags.length === 0 ? 'filled' : 'outlined'} label="All" clickable />
                {props.tagList.map(([tagName, _]) => {
                    return (<Box key={tagName} sx={{ marginLeft: '8px' }}>
                        <Chip onClick={() => handleToggleTag(tagName)} variant={tags.indexOf(tagName) === -1 ? 'outlined' : 'filled'} label={tagName} clickable />
                    </Box>);
                })}
            </Box>
            <Menu
                anchorEl={anchorEl}
                anchorOrigin={{
                    vertical: 'bottom',
                    horizontal: 'left',
                }}
                keepMounted
                transformOrigin={{
                    vertical: 'top',
                    horizontal: 'right',
                }}
                open={Boolean(anchorEl)}
                onClose={handleClose}>
                {menuOptions.map(option => {
                    return (<MenuItem key={option.value} onClick={() => {
                        setSortingOption(option.value);
                        handleClose();
                    }}>
                        {option.value === sortingOption ? <ListItemIcon><Check fontSize="small" /></ListItemIcon> : <></>}
                        {option.label}
                    </MenuItem>
                    )
                })}
            </Menu>
            <Box sx={{ marginLeft: '32px', display: 'flex' }}>
                <IconButton type="button" onClick={handleMenu}>
                    <IconSort />
                </IconButton>
            </Box>
        </Box>
    );
}