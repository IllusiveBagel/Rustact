use std::collections::HashMap;

use crate::runtime::Color;

pub(crate) fn strip_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i += 2;
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}

pub(crate) fn parse_declarations(body: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for declaration in body.split(';') {
        if let Some((name, value)) = declaration.split_once(':') {
            let key = name.trim().to_ascii_lowercase();
            if key.is_empty() {
                continue;
            }
            let value = clean_value(value.trim());
            map.insert(key, value);
        }
    }
    map
}

pub(crate) fn clean_value(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

pub(crate) fn parse_color(value: &str) -> Option<Color> {
    let trimmed = value.trim();
    if trimmed.starts_with('#') {
        let hex = &trimmed[1..];
        return parse_hex_color(hex);
    }
    if let Some(inner) = trimmed
        .strip_prefix("rgb(")
        .and_then(|v| v.strip_suffix(')'))
    {
        let parts: Vec<u8> = inner
            .split(',')
            .filter_map(|part| part.trim().parse::<u8>().ok())
            .collect();
        if parts.len() == 3 {
            return Some(Color::Rgb(parts[0], parts[1], parts[2]));
        }
    }
    named_color(trimmed)
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Color::Rgb(r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        }
        _ => None,
    }
}

fn named_color(value: &str) -> Option<Color> {
    match value.to_ascii_lowercase().as_str() {
        "black" => Some(Color::Black),
        "white" => Some(Color::White),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "blue" => Some(Color::Blue),
        "yellow" => Some(Color::Yellow),
        "cyan" => Some(Color::Cyan),
        "magenta" => Some(Color::Magenta),
        "gray" | "grey" => Some(Color::Gray),
        "lightgray" | "lightgrey" => Some(Color::DarkGray),
        _ => None,
    }
}
