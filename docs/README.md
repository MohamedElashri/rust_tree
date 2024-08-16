# Tree Command Documentation

## Overview
This tree command is a recursive directory listing program that produces a depth-indented listing of files. It includes various display options, sorting capabilities, and filtering features.

## Usage
```
tree [OPTIONS] [PATH]
```
If PATH is not specified, the current directory is used.

## Options

### Display Modes
- `-1, --oneline`: Display one entry per line
- `-l, --long`: Display extended file metadata as a table
- `-G, --grid`: Display entries as a grid
- `-T, --tree`: Recurse into directories as a tree (default)

### Sorting and Traversal
- `--sort <OPTION>`: Sort entries by the specified criteria
  - `name`: Sort by name (default)
  - `size`: Sort by size
  - `time`: Sort by modification time
- `-x, --across`: Sort the grid across, rather than downwards
- `-R, --recurse`: Recurse into directories (applies to non-tree modes)

### Filtering
- `--pattern <REGEX>`: Only show entries that match the given regex pattern
- `--show-hidden`: Show hidden files and directories

### Depth Control
- `--max-depth <N>`: Limit the depth of directory traversal

### File Type Indicators
- `-F, --classify <WHEN>`: Display type indicator by file names
  - `always`: Always show type indicators
  - `auto`: Show type indicators for directories and symlinks (default)
  - `never`: Never show type indicators

### Color Options
- `--color <WHEN>`: When to use terminal colors (always, auto, never)
- `--color-scale <OPTION>`: Highlight levels of 'field' distinctly (all, age, size)
- `--color-scale-mode <MODE>`: Use gradient or fixed colors in --color-scale (fixed, gradient)

### Icons
- `--icons <WHEN>`: When to display icons (always, auto, never)

### File Name Formatting
- `--quote`: Quote file names with spaces (default)
- `--no-quotes`: Don't quote file names with spaces

### Hyperlinks
- `--hyperlink`: Display entries as hyperlinks

### Path Display
- `--absolute <OPTION>`: Display entries with their absolute path (on, follow, off)

### Symbolic Links
- `-X, --dereference`: Dereference symbolic links when displaying information

### File Size
- `--show-size`: Show file sizes

### Screen Width
- `-w, --width <COLS>`: Set screen width in columns

## Examples
1. Display a directory tree with file sizes, sorted by size:
   ```
   tree --show-size --sort size /path/to/directory
   ```

2. Show a long listing of all files (including hidden) that match a pattern:
   ```
   tree -l --show-hidden --pattern ".*\.rs" /path/to/directory
   ```

3. Display a grid view of files, limited to 2 levels deep, with type indicators:
   ```
   tree -G --max-depth 2 -F always /path/to/directory
   ```

4. Show a one-line listing of files, sorted by modification time:
   ```
   tree -1 --sort time /path/to/directory
   ```

5. Display a tree view with dereferenced symbolic links and file sizes:
   ```
   tree -X --show-size /path/to/directory
   ```

6. Show files with color scaling based on age and using gradient mode:
   ```
   tree --color-scale age --color-scale-mode gradient /path/to/directory
   ```

7. Display files as hyperlinks with icons:
   ```
   tree --hyperlink --icons always /path/to/directory
   ```

## Output
The command will display the directory structure according to the specified options. At the end of the output, it will show a summary:
```
N directories, M files
Total size: X.XX UnitB
```
Where:
- N is the total number of directories
- M is the total number of files
- X.XX is the total size of all files
- Unit is the appropriate unit (B, KB, MB, GB, TB, or PB)

## Error Handling
- If a directory cannot be read due to permissions or other issues, an error message will be displayed, and the program will continue with the next entry.
- Invalid options or arguments will result in an error message explaining the issue.

## Notes
- The default display mode is now tree-like, similar to the original tree command.
- Color coding, icons, and other visual enhancements are not enabled by default but can be activated using the appropriate options.
- The grid display adjusts to the terminal width for optimal viewing.
- When using the `--dereference` option, be cautious of circular symbolic links to avoid infinite loops.
- Icons are displayed based on file types when the `--icons` option is set to `always` or `auto`.
- Hyperlinks are created for file names when the `--hyperlink` option is enabled, allowing for clickable links in supporting terminals.
- File names with spaces are quoted by default. Use `--no-quotes` to disable this behavior.
