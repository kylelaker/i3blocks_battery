use std::fmt;
use std::io::Read;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::BatteryError;

#[derive(PartialEq)]
pub enum BatteryStatus {
    CHARGING, DISCHARGING, FULL
}

pub struct Battery {
    pub name: String,
    pub charge_now: u32,
    pub charge_full: u32,
    pub charge_full_design: u32,
    pub cycle_count: u32,
    pub charge_status: BatteryStatus,
    pub current_now: u32,
    pub current_avg: u32,
}

impl FromStr for BatteryStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<BatteryStatus, ()> {
        match s {
            "Charging"    => Ok(BatteryStatus::CHARGING),
            "Discharging" => Ok(BatteryStatus::DISCHARGING),
            "Full"        => Ok(BatteryStatus::FULL),
            _  => Err(()),
        }
    }
}

impl fmt::Display for BatteryStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match *self {
            BatteryStatus::CHARGING    => "Charging",
            BatteryStatus::DISCHARGING => "Discharging",
            BatteryStatus::FULL        => "Full",
        };
        write!(f, "{}", string)
    }
}

impl Battery {
    pub fn initialize(name: &str) -> Result<Battery, BatteryError> {
        let charge_now = read_battery_data(name, "charge_now")?;
        let charge_full = read_battery_data(name, "charge_full")?;
        let charge_full_design = read_battery_data(name, "charge_full_design")?;
        let cycle_count = read_battery_data(name, "cycle_count")?;
        let current_status = read_battery_data(name, "status")?;
        let current_now = read_battery_data(name, "current_now")?;
        let current_avg = read_battery_data(name, "current_avg")?;

        return Ok(Battery {
            name: name.to_owned(),
            charge_now: charge_now,
            charge_full: charge_full,
            charge_full_design: charge_full_design,
            cycle_count: cycle_count,
            charge_status: current_status,
            current_now: current_now,
            current_avg: current_avg,
        });
    }

    pub fn time_remaining(&self) -> String {
        let time_left: f64 = match self.charge_status {
            BatteryStatus::CHARGING => {
                (self.charge_full as f64 - self.charge_now as f64) / self.current_avg as f64
            },
            BatteryStatus::DISCHARGING => {
                (self.charge_now as f64 / self.current_avg as f64)
            },
            _ => 0.0
        };
        let time_left = time_left;
        let hours_left = time_left as u8;
        let mins_left = ((time_left - hours_left as f64) * 60.0) as u8;
        format!("{:02.0}:{:02.0}", hours_left, mins_left)
    }
    pub fn percent_remaining(&self) -> u32 {
        let percent = (self.charge_now * 100) / self.charge_full;
        if percent > 100 {
            return 100;
        }
        return percent;
    }
    pub fn health(&self) -> u32 {
        (self.charge_full * 100) / self.charge_full_design
    }
    pub fn abs_percent_remaining(&self) -> u32 {
        (self.charge_now * 100) / self.charge_full_design
    }
}

fn read_battery_data<T:FromStr>(name: &str, path: &str) -> Result<T, BatteryError> {
   return read_from_file(get_full_path(name, path));
}

fn get_full_path(name: &str, file: &str) -> PathBuf {
    let path: PathBuf = ["/sys", "class", "power_supply", name, file]
        .iter()
        .collect();
    return path;
}

fn read_from_file<T:FromStr>(path: PathBuf) -> Result<T, BatteryError> {
    let mut f = File::open(path.as_path())?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    let dat = match contents.trim().parse::<T>() {
        Ok(data) => data,
        Err(_) => return Err(BatteryError::ConversionError),
    };
    return Ok(dat);
}
