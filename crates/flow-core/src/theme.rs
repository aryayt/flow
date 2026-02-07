use serde::{Deserialize, Serialize};

/// Available themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    #[default]
    Default,
    Claude,
    Twitter,
    NeoBrutalism,
    RetroArcade,
    Aurora,
    Business,
}

impl Theme {
    pub const ALL: &[Self] = &[
        Self::Default,
        Self::Claude,
        Self::Twitter,
        Self::NeoBrutalism,
        Self::RetroArcade,
        Self::Aurora,
        Self::Business,
    ];

    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Claude => "Claude",
            Self::Twitter => "Twitter",
            Self::NeoBrutalism => "Neo Brutalism",
            Self::RetroArcade => "Retro Arcade",
            Self::Aurora => "Aurora",
            Self::Business => "Business",
        }
    }

    #[must_use]
    pub const fn css_class(&self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Claude => "theme-claude",
            Self::Twitter => "theme-twitter",
            Self::NeoBrutalism => "theme-neo-brutalism",
            Self::RetroArcade => "theme-retro-arcade",
            Self::Aurora => "theme-aurora",
            Self::Business => "theme-business",
        }
    }

    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub const fn colors(&self) -> ThemeColors {
        match self {
            Self::Default => ThemeColors {
                primary: AnsiColor(36),
                secondary: AnsiColor(244),
                background: AnsiColor(0),
                foreground: AnsiColor(15),
                accent: AnsiColor(33),
                muted: AnsiColor(240),
                border: AnsiColor(238),
                pending: AnsiColor(226),
                in_progress: AnsiColor(51),
                done: AnsiColor(46),
                blocked: AnsiColor(196),
                error: AnsiColor(196),
                warning: AnsiColor(214),
                css_primary: "#06b6d4",
                css_secondary: "#64748b",
                css_background: "#0f172a",
                css_foreground: "#f8fafc",
            },
            Self::Claude => ThemeColors {
                primary: AnsiColor(208),
                secondary: AnsiColor(180),
                background: AnsiColor(230),
                foreground: AnsiColor(236),
                accent: AnsiColor(215),
                muted: AnsiColor(249),
                border: AnsiColor(223),
                pending: AnsiColor(226),
                in_progress: AnsiColor(51),
                done: AnsiColor(46),
                blocked: AnsiColor(196),
                error: AnsiColor(196),
                warning: AnsiColor(214),
                css_primary: "#d97706",
                css_secondary: "#92400e",
                css_background: "#fffbeb",
                css_foreground: "#1c1917",
            },
            Self::Twitter => ThemeColors {
                primary: AnsiColor(33),
                secondary: AnsiColor(244),
                background: AnsiColor(16),
                foreground: AnsiColor(15),
                accent: AnsiColor(39),
                muted: AnsiColor(240),
                border: AnsiColor(236),
                pending: AnsiColor(226),
                in_progress: AnsiColor(51),
                done: AnsiColor(46),
                blocked: AnsiColor(196),
                error: AnsiColor(196),
                warning: AnsiColor(214),
                css_primary: "#1d9bf0",
                css_secondary: "#71767b",
                css_background: "#000000",
                css_foreground: "#e7e9ea",
            },
            Self::NeoBrutalism => ThemeColors {
                primary: AnsiColor(226),
                secondary: AnsiColor(201),
                background: AnsiColor(15),
                foreground: AnsiColor(16),
                accent: AnsiColor(196),
                muted: AnsiColor(250),
                border: AnsiColor(16),
                pending: AnsiColor(226),
                in_progress: AnsiColor(51),
                done: AnsiColor(46),
                blocked: AnsiColor(196),
                error: AnsiColor(196),
                warning: AnsiColor(214),
                css_primary: "#facc15",
                css_secondary: "#ec4899",
                css_background: "#ffffff",
                css_foreground: "#000000",
            },
            Self::RetroArcade => ThemeColors {
                primary: AnsiColor(46),
                secondary: AnsiColor(51),
                background: AnsiColor(16),
                foreground: AnsiColor(46),
                accent: AnsiColor(201),
                muted: AnsiColor(22),
                border: AnsiColor(28),
                pending: AnsiColor(226),
                in_progress: AnsiColor(51),
                done: AnsiColor(46),
                blocked: AnsiColor(196),
                error: AnsiColor(196),
                warning: AnsiColor(214),
                css_primary: "#22c55e",
                css_secondary: "#06b6d4",
                css_background: "#000000",
                css_foreground: "#22c55e",
            },
            Self::Aurora => ThemeColors {
                primary: AnsiColor(141),
                secondary: AnsiColor(80),
                background: AnsiColor(17),
                foreground: AnsiColor(189),
                accent: AnsiColor(213),
                muted: AnsiColor(60),
                border: AnsiColor(61),
                pending: AnsiColor(226),
                in_progress: AnsiColor(51),
                done: AnsiColor(46),
                blocked: AnsiColor(196),
                error: AnsiColor(196),
                warning: AnsiColor(214),
                css_primary: "#8b5cf6",
                css_secondary: "#14b8a6",
                css_background: "#0f0a2a",
                css_foreground: "#c4b5fd",
            },
            Self::Business => ThemeColors {
                primary: AnsiColor(24),
                secondary: AnsiColor(244),
                background: AnsiColor(255),
                foreground: AnsiColor(235),
                accent: AnsiColor(32),
                muted: AnsiColor(250),
                border: AnsiColor(252),
                pending: AnsiColor(226),
                in_progress: AnsiColor(51),
                done: AnsiColor(46),
                blocked: AnsiColor(196),
                error: AnsiColor(196),
                warning: AnsiColor(214),
                css_primary: "#1e3a5f",
                css_secondary: "#64748b",
                css_background: "#ffffff",
                css_foreground: "#1e293b",
            },
        }
    }

    /// Cycle to the next theme.
    #[must_use]
    pub fn next(&self) -> Self {
        let all = Self::ALL;
        let idx = all.iter().position(|t| t == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }
}

/// ANSI 256-color code.
#[derive(Debug, Clone, Copy)]
pub struct AnsiColor(pub u8);

impl AnsiColor {
    /// ANSI escape for foreground color.
    #[must_use]
    pub fn fg(&self) -> String {
        format!("\x1b[38;5;{}m", self.0)
    }

    /// ANSI escape for background color.
    #[must_use]
    pub fn bg(&self) -> String {
        format!("\x1b[48;5;{}m", self.0)
    }
}

/// Complete color palette for a theme, with both ANSI (TUI) and CSS (web) representations.
#[derive(Debug, Clone)]
pub struct ThemeColors {
    // ANSI colors for TUI
    pub primary: AnsiColor,
    pub secondary: AnsiColor,
    pub background: AnsiColor,
    pub foreground: AnsiColor,
    pub accent: AnsiColor,
    pub muted: AnsiColor,
    pub border: AnsiColor,

    // Status colors (ANSI)
    pub pending: AnsiColor,
    pub in_progress: AnsiColor,
    pub done: AnsiColor,
    pub blocked: AnsiColor,
    pub error: AnsiColor,
    pub warning: AnsiColor,

    // CSS values for web
    pub css_primary: &'static str,
    pub css_secondary: &'static str,
    pub css_background: &'static str,
    pub css_foreground: &'static str,
}

impl ThemeColors {
    /// Generate CSS custom properties for web injection.
    #[must_use]
    pub fn to_css_variables(&self) -> String {
        format!(
            ":root {{\n  --primary: {};\n  --secondary: {};\n  --background: {};\n  --foreground: {};\n}}",
            self.css_primary, self.css_secondary, self.css_background, self.css_foreground
        )
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for Theme {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(Self::Default),
            "claude" => Ok(Self::Claude),
            "twitter" => Ok(Self::Twitter),
            "neo-brutalism" | "neobrutalism" | "neo_brutalism" => Ok(Self::NeoBrutalism),
            "retro-arcade" | "retroarcade" | "retro_arcade" => Ok(Self::RetroArcade),
            "aurora" => Ok(Self::Aurora),
            "business" => Ok(Self::Business),
            _ => Err(format!("unknown theme: {s}")),
        }
    }
}
