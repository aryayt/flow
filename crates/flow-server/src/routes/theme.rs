use crate::{error::AppResult, state::AppState};
use axum::{extract::State, response::Json};
use flow_core::Theme;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeColorsWeb {
    primary: String,
    secondary: String,
    background: String,
    foreground: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeResponse {
    theme: String,
    colors: ThemeColorsWeb,
}

#[derive(Debug, Deserialize)]
pub struct SetThemeRequest {
    theme: String,
}

/// GET /api/theme — Return current theme name and CSS variables
pub async fn get_theme(
    State(_state): State<Arc<AppState>>,
) -> AppResult<Json<ThemeResponse>> {
    // Default to aurora theme for now
    // In the future, this could read from a config file or database
    let theme = Theme::Aurora;
    let theme_colors = theme.colors();

    Ok(Json(ThemeResponse {
        theme: "aurora".to_string(),
        colors: ThemeColorsWeb {
            primary: theme_colors.css_primary.to_string(),
            secondary: theme_colors.css_secondary.to_string(),
            background: theme_colors.css_background.to_string(),
            foreground: theme_colors.css_foreground.to_string(),
        },
    }))
}

/// POST /api/theme — Set theme (accepts {"theme": "aurora"})
pub async fn set_theme(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<SetThemeRequest>,
) -> AppResult<Json<ThemeResponse>> {
    // Parse the theme name
    let theme = match request.theme.as_str() {
        "default" => Theme::Default,
        "claude" => Theme::Claude,
        "twitter" => Theme::Twitter,
        "neo-brutalism" => Theme::NeoBrutalism,
        "retro-arcade" => Theme::RetroArcade,
        "business" => Theme::Business,
        _ => Theme::Aurora, // Default fallback (including "aurora")
    };

    // In the future, persist the theme choice to config/database
    let theme_colors = theme.colors();

    Ok(Json(ThemeResponse {
        theme: request.theme,
        colors: ThemeColorsWeb {
            primary: theme_colors.css_primary.to_string(),
            secondary: theme_colors.css_secondary.to_string(),
            background: theme_colors.css_background.to_string(),
            foreground: theme_colors.css_foreground.to_string(),
        },
    }))
}
