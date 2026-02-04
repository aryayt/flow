use crossterm::style::{Color, Stylize};
use std::time::Duration;

// Color palette - cyberpunk/modern terminal aesthetic
pub mod colors {
    use crossterm::style::Color;

    pub const CYAN: Color = Color::Rgb {
        r: 0,
        g: 255,
        b: 255,
    };
    pub const MAGENTA: Color = Color::Rgb {
        r: 255,
        g: 0,
        b: 255,
    };
    pub const GREEN: Color = Color::Rgb {
        r: 0,
        g: 255,
        b: 136,
    };
    pub const YELLOW: Color = Color::Rgb {
        r: 255,
        g: 255,
        b: 0,
    };
    pub const RED: Color = Color::Rgb {
        r: 255,
        g: 85,
        b: 85,
    };
    pub const BLUE: Color = Color::Rgb {
        r: 100,
        g: 149,
        b: 237,
    };
    pub const DIM: Color = Color::Rgb {
        r: 128,
        g: 128,
        b: 128,
    };
    pub const WHITE: Color = Color::Rgb {
        r: 255,
        b: 255,
        g: 255,
    };
    pub const PURPLE: Color = Color::Rgb {
        r: 189,
        g: 147,
        b: 249,
    };
}

// Icons using Unicode/Nerd Font symbols
pub mod icons {
    pub const FOLDER: &str = "";
    pub const GIT_BRANCH: &str = "";
    pub const TMUX: &str = "";
    pub const CHECK: &str = "";
    pub const CROSS: &str = "";
    pub const ARROW: &str = "";
    pub const DOT: &str = "●";
    pub const CLOCK: &str = "";
    pub const SYNC: &str = "";
    pub const SHIELD: &str = "";
    pub const GAUGE: &str = "";
    pub const WORKTREE: &str = "";
}

/// Format a duration in a human-readable way
#[allow(clippy::cast_precision_loss)]
pub fn format_duration(duration: Duration) -> String {
    let micros = duration.as_micros();
    let millis = duration.as_millis();
    let secs = duration.as_secs_f64();

    if micros < 1000 {
        format!("{micros}μs")
    } else if millis < 1000 {
        format!("{:.2}ms", micros as f64 / 1000.0)
    } else {
        format!("{secs:.2}s")
    }
}

/// Print timing footer
pub fn print_timing(duration: Duration) {
    let time_str = format_duration(duration);
    println!(
        "\n{} {}",
        icons::CLOCK.with(colors::DIM),
        format!("Completed in {time_str}").with(colors::DIM)
    );
}

/// Print a styled header box
pub fn print_header(title: &str) {
    let width = 45;
    let padding = (width - title.len() - 2) / 2;
    let pad_left = " ".repeat(padding);
    let pad_right = " ".repeat(width - title.len() - 2 - padding);

    println!("{}", format!("╭{}╮", "─".repeat(width)).with(colors::CYAN));
    println!(
        "{}{}{}{}{}",
        "│".with(colors::CYAN),
        pad_left.with(colors::CYAN),
        title.bold().with(colors::WHITE),
        pad_right.with(colors::CYAN),
        "│".with(colors::CYAN)
    );
    println!("{}", format!("╰{}╯", "─".repeat(width)).with(colors::CYAN));
}

/// Print a section header
pub fn print_section(icon: &str, title: &str) {
    println!(
        "\n{} {}",
        icon.with(colors::MAGENTA),
        title.bold().with(colors::WHITE)
    );
}

/// Print a key-value pair
pub fn print_kv(key: &str, value: &str) {
    println!(
        "  {} {} {}",
        icons::DOT.with(colors::DIM),
        format!("{key}:").with(colors::DIM),
        value.with(colors::GREEN)
    );
}

/// Print a list item
pub fn print_item(icon: &str, text: &str, color: Color) {
    println!("  {} {}", icon.with(color), text.with(colors::WHITE));
}

/// Print a success message
pub fn print_success(message: &str) {
    println!(
        "{} {}",
        icons::CHECK.with(colors::GREEN),
        message.with(colors::GREEN)
    );
}

/// Print an error message
pub fn print_error(message: &str) {
    println!(
        "{} {}",
        icons::CROSS.with(colors::RED),
        message.with(colors::RED)
    );
}

/// Print a warning message
pub fn print_warning(message: &str) {
    println!(
        "{} {}",
        "⚠".with(colors::YELLOW),
        message.with(colors::YELLOW)
    );
}

/// Print an info message
pub fn print_info(message: &str) {
    println!(
        "{} {}",
        icons::ARROW.with(colors::BLUE),
        message.with(colors::BLUE)
    );
}

/// Print empty state
pub fn print_empty(message: &str) {
    println!("  {}", message.with(colors::DIM).italic());
}

/// Truncate path for display, keeping the important parts
pub fn truncate_path(path: &std::path::Path, max_len: usize) -> String {
    let path_str = path.display().to_string();
    if path_str.len() <= max_len {
        return path_str;
    }

    // Try to show ~/... format
    if let Ok(home) = std::env::var("HOME") {
        if let Some(stripped) = path_str.strip_prefix(&home) {
            let short = format!("~{stripped}");
            if short.len() <= max_len {
                return short;
            }
            // Still too long, truncate from the middle
            let keep = max_len - 3;
            let half = keep / 2;
            return format!("{}...{}", &short[..half], &short[short.len() - half..]);
        }
    }

    // Truncate from beginning
    format!("...{}", &path_str[path_str.len() - max_len + 3..])
}
