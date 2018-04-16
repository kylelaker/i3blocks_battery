use std::io::Read;
use std::fs::File;
use std::process;
use std::str::FromStr;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;


#[derive(PartialEq)]
pub enum BatteryStatus {
    CHARGING, DISCHARGING, FULL
}

pub struct Battery {
    name: String,
    charge_now: u32,
    charge_full: u32,
    charge_full_design: u32,
    cycle_count: u32,
    pub charge_status: BatteryStatus,
    current_now: u32,
    current_avg: u32,
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
    pub fn initialize(name: &str) -> Battery {
        let it = name;
        Battery {
            name: name.to_owned(),
            charge_now: read_from_file(get_full_path(it, "charge_now")),
            charge_full: read_from_file(get_full_path(it, "charge_full")),
            charge_full_design: read_from_file(get_full_path(it, "charge_full_design")),
            cycle_count: read_from_file(get_full_path(it, "cycle_count")),
            charge_status: read_from_file(get_full_path(it, "status")),
            current_now: read_from_file(get_full_path(it, "current_now")),
            current_avg: read_from_file(get_full_path(it, "current_avg")),
        }
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
        format!("{:02.0}{}{:02.0}", hours_left, ":", mins_left)
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

fn get_full_path(name: &str, file: &str) -> String {
    let path: PathBuf = ["/sys", "class", "power_supply", name, file]
        .iter()
        .collect();
    return path.as_path()
        .to_str()
        .unwrap()
        .to_owned();
}

fn read_from_file<T:FromStr>(path: String) -> T where <T as FromStr>::Err: fmt::Debug {
    let mut f = match File::open(path) {
        Ok(f) => f,
        Err(err) => {
            println!("Unable to open. Error: {}", err);
            process::exit(1);
        }
    };
    let mut contents = String::new();
    match f.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(err) => {
            println!("Error reading. Error: {}", err);
            process::exit(1);
        }
    };
    return contents.trim().parse().unwrap();
}
