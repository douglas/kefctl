use std::path::PathBuf;

use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};

#[derive(Debug, Clone)]
pub struct Theme {
    pub fg: Color,
    pub fg_dim: Color,
    pub border: Color,
    pub border_focused: Color,
    pub accent: Color,
    pub accent_secondary: Color,
    pub status_ok: Color,
    pub status_warn: Color,
    pub status_error: Color,
    pub status_demo: Color,
    pub progress_filled: Color,
    pub progress_empty: Color,
    pub badge_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg: Color::White,
            fg_dim: Color::DarkGray,
            border: Color::DarkGray,
            border_focused: Color::Cyan,
            accent: Color::Cyan,
            accent_secondary: Color::Yellow,
            status_ok: Color::Green,
            status_warn: Color::Yellow,
            status_error: Color::Red,
            status_demo: Color::Magenta,
            progress_filled: Color::Cyan,
            progress_empty: Color::DarkGray,
            badge_bg: Color::DarkGray,
        }
    }
}

impl Theme {
    pub fn load() -> Self {
        load_omarchy().unwrap_or_default()
    }

    pub fn block<'a>(&self, title: &'a str, focused: bool) -> Block<'a> {
        if focused {
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(
                    Style::default()
                        .fg(self.border_focused)
                        .add_modifier(Modifier::BOLD),
                )
        } else {
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(self.border))
        }
    }
}

fn omarchy_colors_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".config/omarchy/current/theme/colors.toml")
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

fn load_omarchy() -> Option<Theme> {
    let path = omarchy_colors_path();
    let contents = std::fs::read_to_string(&path).ok()?;
    let table: toml::Table = contents.parse().ok()?;

    let get = |key: &str| -> Option<Color> {
        let val = table.get(key)?.as_str()?;
        parse_hex_color(val)
    };

    let accent = get("accent")?;
    let foreground = get("foreground").unwrap_or(Color::White);
    let color1 = get("color1").unwrap_or(Color::Red);
    let color2 = get("color2").unwrap_or(Color::Green);
    let color3 = get("color3").unwrap_or(Color::Yellow);
    let color8 = get("color8").unwrap_or(Color::DarkGray);

    Some(Theme {
        fg: foreground,
        fg_dim: color8,
        border: color8,
        border_focused: accent,
        accent,
        accent_secondary: color3,
        status_ok: color2,
        status_warn: color3,
        status_error: color1,
        status_demo: Color::Magenta,
        progress_filled: accent,
        progress_empty: color8,
        badge_bg: color8,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color_valid() {
        assert_eq!(parse_hex_color("#89b4fa"), Some(Color::Rgb(0x89, 0xb4, 0xfa)));
        assert_eq!(parse_hex_color("#000000"), Some(Color::Rgb(0, 0, 0)));
        assert_eq!(parse_hex_color("#ffffff"), Some(Color::Rgb(255, 255, 255)));
    }

    #[test]
    fn parse_hex_color_invalid() {
        assert_eq!(parse_hex_color("89b4fa"), None);
        assert_eq!(parse_hex_color("#89b4f"), None);
        assert_eq!(parse_hex_color(""), None);
    }

    #[test]
    fn default_theme_matches_hardcoded() {
        let t = Theme::default();
        assert_eq!(t.fg, Color::White);
        assert_eq!(t.border, Color::DarkGray);
        assert_eq!(t.accent, Color::Cyan);
        assert_eq!(t.status_ok, Color::Green);
    }

    #[test]
    fn load_returns_a_theme() {
        // Should always return a valid theme (Omarchy or default)
        let t = Theme::load();
        // Just verify the struct is populated — actual colors depend
        // on whether Omarchy is installed on this machine
        assert_ne!(format!("{:?}", t.accent), "");
    }
}
