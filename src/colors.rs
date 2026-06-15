use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

pub const RED: &str = "RED";
pub const BLUE: &str = "BLUE";
pub const GREEN: &str = "GREEN";

// List of all color names that can be used for dynamic pieces.
pub const AVAILABLE_COLORS: &[&str] = &[RED, BLUE, GREEN];

fn color_list() -> &'static [(&'static str, LinearRgba)] {
    static COLOR_LIST: OnceLock<Vec<(&'static str, LinearRgba)>> = OnceLock::new();
    COLOR_LIST.get_or_init(|| {
        vec![
            (RED, Color::srgb_u8(216, 46, 63).to_linear()),
            (BLUE, Color::srgb_u8(53, 129, 216).to_linear()),
            (GREEN, Color::srgb_u8(40, 204, 45).to_linear()),
        ]
    })
}

/// Returns a map from color name to LinearRgba.
pub fn get_color_map() -> HashMap<String, LinearRgba> {
    color_list()
        .iter()
        .map(|(name, color)| (name.to_string(), *color))
        .collect()
}

/// Returns the color name (static str) for a given RGBA value, or "UNKNOWN" if not found.
pub fn color_name_from_rgba(rgba: &LinearRgba) -> &'static str {
    let eps = 0.001;
    for (name, color) in color_list() {
        if (rgba.red - color.red).abs() < eps
            && (rgba.green - color.green).abs() < eps
            && (rgba.blue - color.blue).abs() < eps
            && (rgba.alpha - color.alpha).abs() < eps
        {
            return *name;
        }
    }
    "UNKNOWN"
}