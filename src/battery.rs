use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub percent: u8,
    pub charging: bool,
}

pub fn read_battery() -> Option<BatteryInfo> {
    let power_supply = Path::new("/sys/class/power_supply");
    if !power_supply.exists() {
        return None;
    }

    let entries = fs::read_dir(power_supply).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with("BAT") {
            continue;
        }

        let dir = entry.path();

        let capacity = fs::read_to_string(dir.join("capacity")).ok()?;
        let percent = capacity.trim().parse::<u8>().ok()?;

        let status = fs::read_to_string(dir.join("status")).unwrap_or_default();
        let charging = matches!(status.trim(), "Charging" | "Full");

        return Some(BatteryInfo { percent, charging });
    }

    None
}
