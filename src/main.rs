extern crate gtk;
extern crate nix;
mod battery;

use std::env;

use gtk::prelude::*;
use gtk::{ButtonsType, DialogFlags, MessageType, MessageDialog, Window};

use nix::unistd::{fork, ForkResult};

use battery::Battery;
use battery::BatteryStatus;

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

    let mut percent = bat.percent_remaining();

    let (mut icon, mut color) = match percent {
        0  ... 10  => (empty, red),
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

    if bat.charge_status == BatteryStatus::FULL {
        color = green;
        percent = 100;
    }

    let out = format!("<span color=\"{}\" font_desc=\"Font Awesome\">{} </span>{}%\n",
                      color, icon, percent);
    print!("{0:}{0:}", out);
    if percent <= 5 {
        std::process::exit(33);
    }
}

/*
 * Displays the GTK MessageDialog with information about the current status of
 * the battery. This contains all the fields that this program collects
 * information on.
 *
 * This function will return 0 on success and 1 on error.
 */
fn show_gtk_dialog(bat: &Battery) -> i32 {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return 1;
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
    return 0;
}

fn get_env_var(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(val) => {
            if val.is_empty() {
                None
            } else {
                Some(val)
            }
        },
        Err(_) => None
    }
}

fn main() {
    let battery_name = match get_env_var("BLOCK_INSTANCE") {
        Some(val) => val,
        None => "BAT0".to_owned(),
    };

    let bat = Battery::initialize(&battery_name);

    let button: u32 = match get_env_var("BLOCK_BUTTON") {
        Some(val) => val.trim().parse().unwrap(),
        None => 0
    };

    /*
     * For the purposes of i3blocks, 3 corresponds to a right click. When
     * that happens, fork() and then show a dialog. If we don't fork(), then
     * the dialog prevents this process from exiting and the block doesn't
     * update. The printing to the console is done in the parent thread.
     */
    if button == 3 {
        match fork() {
            Ok(ForkResult::Parent { child, .. }) => (),
            Ok(ForkResult::Child) => std::process::exit(show_gtk_dialog(&bat)),
            Err(_) => println!("Fork failed"),
        }
    }
    console_output(&bat);
}
