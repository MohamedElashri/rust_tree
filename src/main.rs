use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::env;
use std::io::{self, Write};
use std::time::SystemTime;
use std::cmp;
use regex::Regex;
use chrono::{DateTime, Local};
use term_size;
use atty;

#[derive(Debug)]
struct Config {
    max_depth: Option<usize>,
    show_hidden: bool,
    root_path: String,
    sort_by: SortBy,
    pattern: Option<Regex>,
    show_size: bool,
    display_mode: DisplayMode,
    classify: Classify,
    dereference: bool,
    color: ColorOption,
    color_scale: Option<ColorScale>,
    color_scale_mode: ColorScaleMode,
    icons: IconOption,
    quote_names: bool,
    hyperlink: bool,
    absolute_path: AbsolutePathOption,
    screen_width: Option<usize>,
    sort_across: bool,
    recurse: bool,
}

#[derive(Debug, Clone, Copy)]
enum SortBy {
    Name,
    Size,
    ModTime,
}

#[derive(Debug, Clone, Copy)]
enum DisplayMode {
    OneLine,
    Long,
    Grid,
    Tree,
}

#[derive(Debug, Clone, Copy)]
enum Classify {
    Always,
    Auto,
    Never,
}

#[derive(Debug, Clone, Copy)]
enum ColorOption {
    Always,
    Auto,
    Never,
}

#[derive(Debug, Clone, Copy)]
enum ColorScale {
    All,
    Age,
    Size,
}

#[derive(Debug, Clone, Copy)]
enum ColorScaleMode {
    Fixed,
    Gradient,
}

#[derive(Debug, Clone, Copy)]
enum IconOption {
    Always,
    Auto,
    Never,
}

#[derive(Debug, Clone, Copy)]
enum AbsolutePathOption {
    On,
    Follow,
    Off,
}

struct TreeStats {
    directories: usize,
    files: usize,
    total_size: u64,
}

struct FileInfo {
    path: PathBuf,
    size: u64,
    mod_time: SystemTime,
    file_type: fs::FileType,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    
    let config = parse_args(&args)?;

    let path = Path::new(&config.root_path);
    let mut stats = TreeStats { directories: 0, files: 0, total_size: 0 };
    
    match config.display_mode {
        DisplayMode::OneLine => {
            let entries = collect_entries(path, &config, &mut stats)?;
            print_entries_oneline(&entries, &config)?;
        },
        DisplayMode::Long => {
            let entries = collect_entries(path, &config, &mut stats)?;
            print_entries_long(&entries, &config)?;
        },
        DisplayMode::Grid => {
            let entries = collect_entries(path, &config, &mut stats)?;
            print_entries_grid(&entries, &config)?;
        },
        DisplayMode::Tree => {
            println!("{}", path.display());
            print_tree(path, 0, &config, &mut stats)?;
        },
    }

    // Print summary
    let summary = format!("\n{} directories, {} files", stats.directories, stats.files);
    let total_size = format!("Total size: {}", format_size(stats.total_size));
    
    // Apply color to summary if enabled
    let (summary, total_size) = if matches!(config.color, ColorOption::Always | ColorOption::Auto) && atty::is(atty::Stream::Stdout) {
        (
            format!("\x1B[1;34m{}\x1B[0m", summary),
            format!("\x1B[1;32m{}\x1B[0m", total_size)
        )
    } else {
        (summary, total_size)
    };

    println!("{}", summary);
    println!("{}", total_size);

    Ok(())
}

fn parse_args(args: &[String]) -> Result<Config, Box<dyn Error>> {
    let mut config = Config {
        max_depth: None,
        show_hidden: false,
        root_path: String::from("."),
        sort_by: SortBy::Name,
        pattern: None,
        show_size: false,
        display_mode: DisplayMode::Tree, // Changed default to Tree
        classify: Classify::Auto,
        dereference: false,
        color: ColorOption::Auto,
        color_scale: None,
        color_scale_mode: ColorScaleMode::Fixed,
        icons: IconOption::Auto,
        quote_names: true,
        hyperlink: false,
        absolute_path: AbsolutePathOption::Off,
        screen_width: None,
        sort_across: false,
        recurse: false,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--max-depth" => {
                i += 1;
                if i < args.len() {
                    config.max_depth = Some(args[i].parse()?);
                } else {
                    return Err("--max-depth requires a value".into());
                }
            }
            "--show-hidden" => config.show_hidden = true,
            "--sort" => {
                i += 1;
                if i < args.len() {
                    config.sort_by = match args[i].as_str() {
                        "name" => SortBy::Name,
                        "size" => SortBy::Size,
                        "time" => SortBy::ModTime,
                        _ => return Err("Invalid sort option".into()),
                    };
                } else {
                    return Err("--sort requires a value".into());
                }
            }
            "--pattern" => {
                i += 1;
                if i < args.len() {
                    config.pattern = Some(Regex::new(&args[i])?);
                } else {
                    return Err("--pattern requires a value".into());
                }
            }
            "--show-size" => config.show_size = true,
            "-1" | "--oneline" => config.display_mode = DisplayMode::OneLine,
            "-l" | "--long" => config.display_mode = DisplayMode::Long,
            "-G" | "--grid" => config.display_mode = DisplayMode::Grid,
            "-T" | "--tree" => config.display_mode = DisplayMode::Tree,
            "-X" | "--dereference" => config.dereference = true,
            "-F" | "--classify" => {
                i += 1;
                if i < args.len() {
                    config.classify = match args[i].as_str() {
                        "always" => Classify::Always,
                        "auto" => Classify::Auto,
                        "never" => Classify::Never,
                        _ => return Err("Invalid classify option".into()),
                    };
                } else {
                    return Err("--classify requires a value".into());
                }
            }
            "--color" | "--colour" => {
                i += 1;
                if i < args.len() {
                    config.color = match args[i].as_str() {
                        "always" => ColorOption::Always,
                        "auto" => ColorOption::Auto,
                        "never" => ColorOption::Never,
                        _ => return Err("Invalid color option".into()),
                    };
                } else {
                    return Err("--color requires a value".into());
                }
            }
            "--color-scale" | "--colour-scale" => {
                i += 1;
                if i < args.len() {
                    config.color_scale = Some(match args[i].as_str() {
                        "all" => ColorScale::All,
                        "age" => ColorScale::Age,
                        "size" => ColorScale::Size,
                        _ => return Err("Invalid color scale option".into()),
                    });
                } else {
                    return Err("--color-scale requires a value".into());
                }
            }
            "--color-scale-mode" | "--colour-scale-mode" => {
                i += 1;
                if i < args.len() {
                    config.color_scale_mode = match args[i].as_str() {
                        "fixed" => ColorScaleMode::Fixed,
                        "gradient" => ColorScaleMode::Gradient,
                        _ => return Err("Invalid color scale mode".into()),
                    };
                } else {
                    return Err("--color-scale-mode requires a value".into());
                }
            }
            "--icons" => {
                i += 1;
                if i < args.len() {
                    config.icons = match args[i].as_str() {
                        "always" => IconOption::Always,
                        "auto" => IconOption::Auto,
                        "never" => IconOption::Never,
                        _ => return Err("Invalid icons option".into()),
                    };
                } else {
                    return Err("--icons requires a value".into());
                }
            }
            "--no-quotes" => config.quote_names = false,
            "--hyperlink" => config.hyperlink = true,
            "--absolute" => {
                i += 1;
                if i < args.len() {
                    config.absolute_path = match args[i].as_str() {
                        "on" => AbsolutePathOption::On,
                        "follow" => AbsolutePathOption::Follow,
                        "off" => AbsolutePathOption::Off,
                        _ => return Err("Invalid absolute path option".into()),
                    };
                } else {
                    return Err("--absolute requires a value".into());
                }
            }
            "-w" | "--width" => {
                i += 1;
                if i < args.len() {
                    config.screen_width = Some(args[i].parse()?);
                } else {
                    return Err("--width requires a value".into());
                }
            }
            "-x" | "--across" => config.sort_across = true,
            "-R" | "--recurse" => config.recurse = true,
            _ => {
                config.root_path = args[i].clone();
            }
        }
        i += 1;
    }

    Ok(config)
}

// Include all other functions from your original implementation here
// This includes collect_entries, print_entries_oneline, print_entries_long, print_entries_grid, print_tree, etc.

fn collect_entries(path: &Path, config: &Config, stats: &mut TreeStats) -> io::Result<Vec<FileInfo>> {
    let mut entries = Vec::new();

    if path.is_dir() {
        stats.directories += 1;
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            
            if !config.show_hidden && is_hidden(&path) {
                continue;
            }

            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            if let Some(pattern) = &config.pattern {
                if !pattern.is_match(&file_name) && !path.is_dir() {
                    continue;
                }
            }

            let metadata = if config.dereference {
                fs::metadata(&path)?
            } else {
                entry.metadata()?
            };

            let file_info = FileInfo {
                path: get_display_path(&path, config),
                size: metadata.len(),
                mod_time: metadata.modified()?,
                file_type: metadata.file_type(),
            };

            stats.total_size += file_info.size;

            if path.is_file() {
                stats.files += 1;
            }

            entries.push(file_info);

            if config.recurse && path.is_dir() {
                let mut sub_entries = collect_entries(&path, config, stats)?;
                entries.append(&mut sub_entries);
            }
        }
    }

    sort_entries(&mut entries, config.sort_by);

    Ok(entries)
}

fn print_entries_oneline(entries: &[FileInfo], config: &Config) -> io::Result<()> {
    for entry in entries {
        print_entry_oneline(entry, config)?;
    }
    Ok(())
}

fn print_entry_oneline(entry: &FileInfo, config: &Config) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    let file_name = entry.path.file_name().unwrap_or_default().to_string_lossy();
    let formatted_name = format_file_name(&file_name, config);
    let hyperlinked_name = format_hyperlink(&entry.path, &formatted_name, config);
    let icon = get_icon(&entry.path, config);
    let color = get_color_for_scale(&entry.path, config);
    let type_indicator = get_type_indicator(&entry.file_type, config.classify);
    
    write!(stdout, "{}{}{}{}", color, icon, hyperlinked_name, type_indicator)?;
    
    if config.show_size {
        write!(stdout, " [{}]", format_size(entry.size))?;
    }
    
    writeln!(stdout, "\x1B[0m")
}

fn print_entries_long(entries: &[FileInfo], config: &Config) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    
    // Calculate column widths
    let max_size_width = entries.iter().map(|e| format_size(e.size).len()).max().unwrap_or(0);
    let max_name_width = entries.iter().map(|e| e.path.file_name().unwrap_or_default().len()).max().unwrap_or(0);

    // Print header
    writeln!(stdout, "{:<10} {:>width$} {:<20} {}",
        "Type",
        "Size",
        "Modified",
        "Name",
        width = max_size_width
    )?;
    writeln!(stdout, "{}", "-".repeat(10 + 1 + max_size_width + 1 + 20 + 1 + max_name_width))?;

    for entry in entries {
        print_entry_long(entry, config, max_size_width)?;
    }

    Ok(())
}

fn print_entry_long(entry: &FileInfo, config: &Config, size_width: usize) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    let file_name = entry.path.file_name().unwrap_or_default().to_string_lossy();
    let formatted_name = format_file_name(&file_name, config);
    let hyperlinked_name = format_hyperlink(&entry.path, &formatted_name, config);
    let icon = get_icon(&entry.path, config);
    let color = get_color_for_scale(&entry.path, config);
    let type_indicator = get_type_indicator(&entry.file_type, config.classify);
    let size = format_size(entry.size);
    let mod_time: DateTime<Local> = entry.mod_time.into();

    writeln!(stdout, "{}{:<10} {:>width$} {:<20} {}{}{}{}{}",
        color,
        get_file_type_str(&entry.file_type),
        size,
        mod_time.format("%Y-%m-%d %H:%M:%S"),
        icon,
        hyperlinked_name,
        type_indicator,
        if config.show_size { format!(" [{}]", size) } else { String::new() },
        "\x1B[0m",
        width = size_width
    )
}

fn print_entries_grid(entries: &[FileInfo], config: &Config) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    let term_width = config.screen_width.unwrap_or_else(|| term_size::dimensions().map(|(w, _)| w).unwrap_or(80));
    
    let max_entry_width = entries.iter()
        .map(|e| {
            let file_name = e.path.file_name().unwrap_or_default().to_string_lossy();
            let formatted_name = format_file_name(&file_name, config);
            let icon = get_icon(&e.path, config);
            let type_indicator = get_type_indicator(&e.file_type, config.classify);
            let size_str = if config.show_size { format!(" [{}]", format_size(e.size)) } else { String::new() };
            icon.len() + formatted_name.len() + type_indicator.len() + size_str.len()
        })
        .max()
        .unwrap_or(0) + 2;  // +2 for spacing between entries

    let columns = term_width / max_entry_width;
    let rows = (entries.len() + columns - 1) / columns;

    for row in 0..rows {
        for col in 0..columns {
            let index = if config.sort_across {
                row * columns + col
            } else {
                col * rows + row
            };

            if index < entries.len() {
                let entry = &entries[index];
                print_entry_grid(entry, config, max_entry_width)?;
            }
        }
        writeln!(stdout)?;
    }

    Ok(())
}

fn print_entry_grid(entry: &FileInfo, config: &Config, width: usize) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    let file_name = entry.path.file_name().unwrap_or_default().to_string_lossy();
    let formatted_name = format_file_name(&file_name, config);
    let hyperlinked_name = format_hyperlink(&entry.path, &formatted_name, config);
    let icon = get_icon(&entry.path, config);
    let color = get_color_for_scale(&entry.path, config);
    let type_indicator = get_type_indicator(&entry.file_type, config.classify);
    
    let size_str = if config.show_size { 
        format!(" [{}]", format_size(entry.size)) 
    } else { 
        String::new() 
    };
    
    let entry_str = format!("{}{}{}{}{}", icon, hyperlinked_name, type_indicator, size_str, "\x1B[0m");
    
    write!(stdout, "{}{:<width$}", color, entry_str, width = width)
}

fn print_tree(path: &Path, level: usize, config: &Config, stats: &mut TreeStats) -> io::Result<()> {
    if let Some(max_depth) = config.max_depth {
        if level >= max_depth {
            return Ok(());
        }
    }

    let display_path = get_display_path(path, config);

    if level > 0 {
        let prefix = if level == 1 {
            "├── ".to_string()
        } else {
            format!("{}├── ", "│   ".repeat(level - 1))
        };

        print_tree_entry(&display_path, &prefix, config)?;
    }

    if display_path.is_dir() {
        stats.directories += 1;
        let mut entries: Vec<_> = fs::read_dir(&display_path)?
            .filter_map(Result::ok)
            .filter(|e| config.show_hidden || !is_hidden(&e.path()))
            .collect();

        sort_entries_by_path(&mut entries, config.sort_by);

        let total_entries = entries.len();
        for (index, entry) in entries.iter().enumerate() {
            let is_last = index == total_entries - 1;
            
            if is_last && level > 0 {
                print!("{}└── ", "│   ".repeat(level - 1));
            }

            print_tree(&entry.path(), level + 1, config, stats)?;

            if is_last && level > 0 {
                print!("{}    ", "    ".repeat(level - 1));
            }
        }
    } else {
        stats.files += 1;
        let metadata = fs::metadata(&display_path)?;
        stats.total_size += metadata.len();
    }

    Ok(())
}

fn print_tree_entry(path: &Path, prefix: &str, config: &Config) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    let formatted_name = format_file_name(&file_name, config);
    let hyperlinked_name = format_hyperlink(path, &formatted_name, config);
    let icon = get_icon(path, config);
    let color = get_color_for_scale(path, config);
    let type_indicator = get_type_indicator(&fs::metadata(path)?.file_type(), config.classify);

    write!(stdout, "{}", prefix)?;
    write!(stdout, "{}{}{}{}\x1B[0m", color, icon, hyperlinked_name, type_indicator)?;

    if config.show_size {
        let size = fs::metadata(path)?.len();
        write!(stdout, " [{}]", format_size(size))?;
    }

    writeln!(stdout)
}

fn sort_entries(entries: &mut Vec<FileInfo>, sort_by: SortBy) {
    match sort_by {
        SortBy::Name => entries.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name())),
        SortBy::Size => entries.sort_by(|a, b| b.size.cmp(&a.size)),
        SortBy::ModTime => entries.sort_by(|a, b| b.mod_time.cmp(&a.mod_time)),
    }
}

fn sort_entries_by_path(entries: &mut Vec<fs::DirEntry>, sort_by: SortBy) {
    match sort_by {
        SortBy::Name => entries.sort_by(|a, b| a.file_name().cmp(&b.file_name())),
        SortBy::Size => entries.sort_by(|a, b| b.metadata().map(|m| m.len()).unwrap_or(0)
                                         .cmp(&a.metadata().map(|m| m.len()).unwrap_or(0))),
        SortBy::ModTime => entries.sort_by(|a, b| b.metadata().and_then(|m| m.modified()).unwrap_or_else(|_| SystemTime::now())
                                            .cmp(&a.metadata().and_then(|m| m.modified()).unwrap_or_else(|_| SystemTime::now()))),
    }
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with("."))
        .unwrap_or(false)
}

fn get_color_for_scale(path: &Path, config: &Config) -> String {
    match config.color_scale {
        Some(ColorScale::Age) => get_color_for_age(path, config),
        Some(ColorScale::Size) => get_color_for_size(path, config),
        Some(ColorScale::All) => {
            let age_color = get_color_for_age(path, config);
            let size_color = get_color_for_size(path, config);
            format!("{};{}", age_color, size_color)
        },
        None => String::new(),
    }
}

fn get_color_for_age(path: &Path, config: &Config) -> String {
    let metadata = fs::metadata(path).unwrap();
    let age = SystemTime::now().duration_since(metadata.modified().unwrap()).unwrap().as_secs();
    
    match config.color_scale_mode {
        ColorScaleMode::Fixed => {
            if age < 60 * 60 * 24 { // 1 day
                "\x1B[38;5;46m".to_string() // Bright green
            } else if age < 60 * 60 * 24 * 7 { // 1 week
                "\x1B[38;5;226m".to_string() // Yellow
            } else if age < 60 * 60 * 24 * 30 { // 1 month
                "\x1B[38;5;208m".to_string() // Orange
            } else {
                "\x1B[38;5;196m".to_string() // Red
            }
        },
        ColorScaleMode::Gradient => {
            let max_age = 60 * 60 * 24 * 365; // 1 year
            let normalized_age = (age as f32 / max_age as f32).min(1.0);
            let hue = (1.0 - normalized_age) * 120.0; // 120 (green) to 0 (red)
            let (r, g, b) = hue_to_rgb(hue);
            format!("\x1B[38;2;{};{};{}m", r, g, b)
        },
    }
}

fn get_color_for_size(path: &Path, config: &Config) -> String {
    let size = fs::metadata(path).unwrap().len();
    
    match config.color_scale_mode {
        ColorScaleMode::Fixed => {
            if size < 1024 { // 1 KB
                "\x1B[38;5;46m".to_string() // Bright green
            } else if size < 1024 * 1024 { // 1 MB
                "\x1B[38;5;226m".to_string() // Yellow
            } else if size < 1024 * 1024 * 100 { // 100 MB
                "\x1B[38;5;208m".to_string() // Orange
            } else {
                "\x1B[38;5;196m".to_string() // Red
            }
        },
        ColorScaleMode::Gradient => {
            let max_size = 1024 * 1024 * 1024; // 1 GB
            let normalized_size = (size as f32 / max_size as f32).min(1.0);
            let hue = (1.0 - normalized_size) * 120.0; // 120 (green) to 0 (red)
            let (r, g, b) = hue_to_rgb(hue);
            format!("\x1B[38;2;{};{};{}m", r, g, b)
        },
    }
}

fn hue_to_rgb(hue: f32) -> (u8, u8, u8) {
    let c = 1.0;
    let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let m = 0.0;

    let (r, g, b) = match hue {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8
    )
}

fn get_icon(path: &Path, config: &Config) -> &'static str {
    match config.icons {
        IconOption::Always => get_icon_for_file(path),
        IconOption::Auto => {
            if atty::is(atty::Stream::Stdout) {
                get_icon_for_file(path)
            } else {
                ""
            }
        },
        IconOption::Never => "",
    }
}

fn get_icon_for_file(path: &Path) -> &'static str {
    if path.is_dir() {
        "📁 "
    } else {
        match path.extension().and_then(|s| s.to_str()) {
            Some("txt") => "📄 ",
            Some("rs") => "🦀 ",
            Some("py") => "🐍 ",
            Some("js") => "🟨 ",
            Some("html") => "🌐 ",
            Some("css") => "🎨 ",
            Some("json") => "🔧 ",
            Some("md") => "📝 ",
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") => "🖼️ ",
            Some("mp3") | Some("wav") | Some("ogg") => "🎵 ",
            Some("mp4") | Some("avi") | Some("mkv") => "🎥 ",
            Some("pdf") => "📚 ",
            Some("zip") | Some("tar") | Some("gz") => "🗜️ ",
            Some("exe") => "⚙️ ",
            _ => "📄 ",
        }
    }
}

fn format_file_name(name: &str, config: &Config) -> String {
    if config.quote_names && name.contains(' ') {
        format!("\"{}\"", name)
    } else {
        name.to_string()
    }
}


fn format_hyperlink(path: &Path, name: &str, config: &Config) -> String {
    if config.hyperlink {
        let full_path = if path.is_absolute() {
            path.to_string_lossy().to_string()
        } else {
            env::current_dir().unwrap().join(path).to_string_lossy().to_string()
        };
        format!("\x1B]8;;file://{}\x1B\\{}\x1B]8;;\x1B\\", full_path, name)
    } else {
        name.to_string()
    }
}

fn get_display_path(path: &Path, config: &Config) -> PathBuf {
    match config.absolute_path {
        AbsolutePathOption::On => path.canonicalize().unwrap_or_else(|_| path.to_path_buf()),
        AbsolutePathOption::Follow => {
            if path.is_symlink() {
                fs::read_link(path).unwrap_or_else(|_| path.to_path_buf())
            } else {
                path.to_path_buf()
            }
        },
        AbsolutePathOption::Off => path.to_path_buf(),
    }
}

fn get_type_indicator(file_type: &fs::FileType, classify: Classify) -> &'static str {
    match classify {
        Classify::Always => {
            if file_type.is_dir() { "/" }
            else if file_type.is_symlink() { "@" }
            else if file_type.is_file() { 
                #[cfg(unix)]
                {
                    use std::os::unix::fs::FileTypeExt;
                    if file_type.is_socket() { "=" }
                    else if file_type.is_fifo() { "|" }
                    else { "" }
                }
                #[cfg(not(unix))]
                { "" }
            }
            else { "" }
        },
        Classify::Auto => {
            if file_type.is_dir() { "/" }
            else if file_type.is_symlink() { "@" }
            else { "" }
        },
        Classify::Never => "",
    }
}

fn get_file_type_str(file_type: &fs::FileType) -> &'static str {
    if file_type.is_dir() { "Directory" }
    else if file_type.is_symlink() { "Symlink" }
    else if file_type.is_file() { "File" }
    else { "Other" }
}

fn format_size(size: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(path) {
        return metadata.permissions().mode() & 0o111 != 0;
    }
    false
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool {
    false
}



