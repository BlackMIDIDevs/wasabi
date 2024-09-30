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
