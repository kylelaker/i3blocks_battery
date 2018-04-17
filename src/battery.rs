use std::io::Read;
use std::fs::File;
use std::process;
use std::str::FromStr;
use std::fmt;
use std::path::PathBuf;


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
    pub fn initialize(name: &str) -> Option<Battery> {
        let charge_now = match read_battery_data(name, "charge_now") {
            Some(data) => data,
            None => return None,
        };
        let charge_full = match read_battery_data(name, "charge_full") {
            Some(data) => data,
            None => return None,
        };
        let charge_full_design = match read_battery_data(name, "charge_full_design") {
            Some(data) => data,
            None => return None,
        };
        let cycle_count = match read_battery_data(name, "cycle_count") {
            Some(data) => data,
            None => return None,
        };
        let current_status = match read_battery_data(name, "status") {
            Some(data) => data,
            None => return None,
        };
        let current_now = match read_battery_data(name, "current_now") {
            Some(data) => data,
            None => return None,
        };
        let current_avg = match read_battery_data(name, "current_avg") {
            Some(data) => data,
            None => return None,
        };
        return Some(Battery {
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
        (self.charge_now * 100) / self.charge_full
    }
    pub fn health(&self) -> u32 {
        (self.charge_full * 100) / self.charge_full_design
    }
    pub fn abs_percent_remaining(&self) -> u32 {
        (self.charge_now * 100) / self.charge_full_design
    }
}

fn read_battery_data<T:FromStr>(name: &str, path: &str) -> Option<T> where <T as FromStr>::Err: fmt::Debug {
   return read_from_file(get_full_path(name, path));
}

fn get_full_path(name: &str, file: &str) -> PathBuf {
    let path: PathBuf = ["/sys", "class", "power_supply", name, file]
        .iter()
        .collect();
    return path;
}

fn read_from_file<T:FromStr>(path: PathBuf) -> Option<T> where <T as FromStr>::Err: fmt::Debug {
    let mut f = match File::open(path.as_path()) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Unable to open {:?}. Error: {}", path, err);
            return None;
        }
    };

    let mut contents = String::new();
    match f.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Error reading. Error: {}", err);
            return None;
        }
    };
    return match contents.trim().parse::<T>() {
        Ok(val) => Some(val),
        Err(_) => None,
    }
}
