extern crate gtk;
extern crate libnotify;
extern crate nix;

mod battery;
mod error;

use std::env;

use gtk::prelude::*;
use gtk::{ButtonsType, DialogFlags, MessageType, MessageDialog, Window};

use nix::unistd::{fork, ForkResult};

use crate::battery::Battery;
use crate::battery::BatteryStatus;
use crate::error::BatteryError;

const LEFT_CLICK: i32 = 1;
const RIGHT_CLICK: i32 = 3;
const CRITICAL_EXIT_CODE: i32 = 33;
const CRITICAL_BAT_LEVEL: u32 = 5;

fn console_output(bat: &Battery) {
    // Icons
    let plug          = '\u{f1e6}';
    let check         = '\u{f00c}';
    let full          = '\u{f240}';
    let three_quarter = '\u{f241}';
    let half          = '\u{f242}';
    let quarter       = '\u{f243}';
    let empty         = '\u{f244}';

    // Colors
    let green  = "#859900";
    let cyan   = "#2AA198";
    let yellow = "#B58900";
    let orange = "#CB4B16";
    let red    = "#DC322F";
    let bg     = "#073642";

    let mut percent = bat.percent_remaining();

    let (mut icon, mut color) = match percent {
        0  ... 5   => (empty, bg),
        6  ... 10  => (empty, red),
        11 ... 35  => (quarter, orange),
        36 ... 60  => (half, yellow),
        61 ... 85  => (three_quarter, cyan),
        86 ... 100 => (full, green),
        _          => std::process::exit(1),
    };

    match bat.charge_status {
        BatteryStatus::FULL => {
            icon = check;
            color = green;
            percent = 100;
        },
        BatteryStatus::CHARGING => {
            icon = plug;
        },
        _ => (),
    };

    let out = format!("<span color=\"{}\" font_desc=\"Font Awesome\"> {} </span>{}%\n",
                      color, icon, percent);
    print!("{0:}{0:}", out);

}

/*
 * Displays the GTK MessageDialog with information about the current status of
 * the battery. This contains all the fields that this program collects
 * information on.
 *
 * This function will return 0 on success and 1 on error.
 */
fn show_gtk_dialog(bat: &Battery) -> Result<(), &'static str> {
    if gtk::init().is_err() {
        return Err("Failed to initialize GTK");
    }

    let body_string = format!("Battery name:\t\t{}\n\
                              Battery charge:\t\t{}\n\
                              Charge when full:\t{}\n\
                              Design full:\t\t\t{}\n\
                              Cycle count:\t\t{}\n\
                              Charging status:\t{}\n\
                              Current now:\t\t{}\n\
                              Current avg:\t\t{}\n\
                              % Charged:\t\t\t{}%\n\
                              Time remaining:\t\t{}\n\
                              Battery health:\t\t{}%\n\
                              % Charged (abs):\t{}%\n",
                              bat.name, bat.charge_now, bat.charge_full,
                              bat.charge_full_design, bat.cycle_count,
                              bat.charge_status, bat.current_now,
                              bat.current_avg, bat.percent_remaining(),
                              bat.time_remaining(), bat.health(),
                              bat.abs_percent_remaining());

    MessageDialog::new(None::<&Window>,
                       DialogFlags::empty(),
                       MessageType::Info,
                       ButtonsType::Ok,
                       &body_string).run();
    return Ok(());
}

fn get_env_var(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(ref val) if val.is_empty() => None,
        Ok(val) => Some(val),
        Err(_) => None
    }
}

fn show_notification(bat: &Battery) {
    let time_left = bat.time_remaining();
    libnotify::init("Battery").unwrap();
    let notification = libnotify::Notification::new("Time Remaining",
                                                    Some(&*time_left),
                                                    None);
    notification.show().unwrap();
    libnotify::uninit();
}

/*
 * For the purposes of i3blocks, 3 corresponds to a right click. When
 * that happens, fork() and then show a dialog. If we don't fork(), then
 * the dialog prevents this process from exiting and the block doesn't
 * update. The printing to the console is done in the parent thread.
 */
fn handle_button_presses(bat: &Battery) {
    let button: i32 = match get_env_var("BLOCK_BUTTON") {
        Some(val) => val.trim().parse().unwrap(),
        None => 0
    };

    match button {
        RIGHT_CLICK => {
            match fork() {
                Ok(ForkResult::Parent{ .. }) => (),
                Ok(ForkResult::Child) => {
                    match show_gtk_dialog(&bat) {
                        Ok(_) => std::process::exit(0),
                        Err(_) => std::process::exit(1),
                    }
                }
                Err(_) => eprintln!("Fork failed"),
            }
        },
        LEFT_CLICK => show_notification(&bat),
        _ => (),
    };
    console_output(&bat);
}

fn main() -> Result<(), BatteryError>{
    let battery_name = match get_env_var("BLOCK_INSTANCE") {
        Some(val) => val,
        None => "BAT0".to_owned(),
    };

    let bat = Battery::initialize(&battery_name)?;

    handle_button_presses(&bat);

    /*
     * i3blocks shows a block as critical if the process exits with code 33.
     * If the battery is less than 33% charged, the status is critical
     */
    if bat.percent_remaining() <= CRITICAL_BAT_LEVEL {
        std::process::exit(CRITICAL_EXIT_CODE);
    }

    return Ok(());
}
