use reqwest::blocking::ClientBuilder;
use serde_json::Value;
use std::{collections::HashMap, ops::RangeInclusive};

use crate::{gui::window::WasabiError, state::WasabiState};

pub const NOTE_SPEED_RANGE: RangeInclusive<f64> = 8.0..=0.05;

pub fn calculate_border_width(width_pixels: f32, keys_len: f32) -> f32 {
    ((width_pixels / keys_len) / 12.0).clamp(1.0, 5.0).round() * 2.0
}

pub fn convert_seconds_to_time_string(sec: f64) -> String {
    let time_millis = (sec * 10.0) as i64 % 10;
    let time_sec = sec as i64 % 60;
    let time_min = sec as i64 / 60;

    format!(
        "{}{:0width$}:{:0width$}.{}",
        if time_sec + time_millis < 0 {
            '-'
        } else {
            '\0'
        },
        time_min.abs(),
        time_sec.abs(),
        time_millis.abs(),
        width = 2
    )
}

fn get_latest_version() -> Result<String, WasabiError> {
    let api_url = "https://api.github.com/repos/BlackMIDIDevs/wasabi/releases/latest";
    let current = format!("v{}", env!("CARGO_PKG_VERSION"));

    let client = ClientBuilder::new()
        .user_agent("Wasabi_Updater")
        .build()
        .map_err(|e| WasabiError::UpdaterError(e.to_string()))?;
    let data = client
        .get(api_url)
        .send()
        .map_err(|e| WasabiError::UpdaterError(e.to_string()))?;
    let txt = data.text().unwrap_or_default();
    let json = serde_json::from_str::<HashMap<String, Value>>(&txt)
        .map_err(|e| WasabiError::UpdaterError(e.to_string()))?;

    Ok(if let Some(tag) = json.get("tag_name") {
        tag.as_str().unwrap_or(&current).to_owned()
    } else {
        current.into()
    })
}
pub fn check_for_updates(state: &WasabiState) {
    let current = format!("v{}", env!("CARGO_PKG_VERSION"));
    match get_latest_version() {
        Ok(latest) => {
            if latest != current {
                state.errors.new_update(latest);
            }
        }
        Err(e) => state.errors.error(&e),
    }
}
