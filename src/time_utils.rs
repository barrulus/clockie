use chrono::{Local, Timelike};
use chrono_tz::Tz;

#[derive(Debug, Clone)]
pub struct ClockTime {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub hour12: u32,
    pub is_pm: bool,
    pub date_string: String,
}

impl ClockTime {
    pub fn format_time(&self, hour_format: u8, show_seconds: bool) -> String {
        let h = if hour_format == 12 { self.hour12 } else { self.hour };
        if show_seconds {
            format!("{:02}:{:02}:{:02}", h, self.minute, self.second)
        } else {
            format!("{:02}:{:02}", h, self.minute)
        }
    }

    pub fn format_time_suffix(&self, hour_format: u8) -> &'static str {
        if hour_format == 12 {
            if self.is_pm { " PM" } else { " AM" }
        } else {
            ""
        }
    }
}

pub fn current_time(date_format: &str) -> ClockTime {
    let now = Local::now();
    let hour = now.hour();
    let hour12 = if hour == 0 { 12 } else if hour > 12 { hour - 12 } else { hour };
    ClockTime {
        hour,
        minute: now.minute(),
        second: now.second(),
        hour12,
        is_pm: hour >= 12,
        date_string: now.format(date_format).to_string(),
    }
}

pub fn timezone_time(tz_str: &str, hour_format: u8, show_seconds: bool) -> Option<String> {
    let tz: Tz = tz_str.parse().ok()?;
    let now = chrono::Utc::now().with_timezone(&tz);
    let hour = now.hour();
    let h = if hour_format == 12 {
        if hour == 0 { 12 } else if hour > 12 { hour - 12 } else { hour }
    } else {
        hour
    };
    let suffix = if hour_format == 12 {
        if hour >= 12 { " PM" } else { " AM" }
    } else {
        ""
    };
    if show_seconds {
        Some(format!("{:02}:{:02}:{:02}{}", h, now.minute(), now.second(), suffix))
    } else {
        Some(format!("{:02}:{:02}{}", h, now.minute(), suffix))
    }
}
