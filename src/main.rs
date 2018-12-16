extern crate gtk;
extern crate libnotify;
extern crate nix;

mod battery;
mod error;

use std::env;

use gtk::prelude::*;
use gtk::{Align, ButtonsType, DialogFlags, Grid, Label, MessageType, MessageDialog, Window};

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

/// Creates a left-aligned GTK Label
fn create_label(content: &str) -> Label {
    let label = Label::new(Some(content));
    label.set_halign(Align::Start);
    return label;
}

/// Displays the GTK MessageDialog with information about the current status of
/// the battery. This contains all the fields that this program collects
/// information on.
/// This function will return 0 on success and 1 on error.
fn show_gtk_dialog(bat: &Battery) -> Result<(), &'static str> {
    if gtk::init().is_err() {
        return Err("Failed to initialize GTK");
    }

    let grid = Grid::new();

    let bat_name = format!("{}", bat.name);
    let bat_charge = format!("{} Ah", bat.charge_now);
    let charge_full = format!("{} Ah", bat.charge_full);
    let design_full = format!("{} Ah", bat.charge_full_design);
    let cycle_count = format!("{} cycles", bat.cycle_count);
    let charge_status = format!("{}", bat.charge_status);
    let current_now = format!("{} Ah", bat.current_now);
    let current_avg = format!("{} Ah", bat.current_avg);
    let percent = format!("{}%", bat.percent_remaining());
    let time = format!("{} hrs", bat.time_remaining());
    let health = format!("{}%", bat.health());
    let abs_percent = format!("{}%", bat.abs_percent_remaining());

    grid.attach(&create_label("Battery name:"), 0, 0, 1, 1);
    grid.attach(&create_label(bat_name.as_str()), 1, 0, 1, 1);
    grid.attach(&create_label("Battery charge:"), 0, 1, 1, 1);
    grid.attach(&create_label(bat_charge.as_str()), 1, 1, 1, 1);
    grid.attach(&create_label("Charge when full:"), 0, 2, 1, 1);
    grid.attach(&create_label(charge_full.as_str()), 1, 2, 1, 1);
    grid.attach(&create_label("Design full:"), 0, 3, 1, 1);
    grid.attach(&create_label(design_full.as_str()), 1, 3, 1, 1);
    grid.attach(&create_label("Cycle count:"), 0, 4, 1, 1);
    grid.attach(&create_label(cycle_count.as_str()), 1, 4, 1, 1);
    grid.attach(&create_label("Status:"), 0, 5, 1, 1);
    grid.attach(&create_label(charge_status.as_str()), 1, 5, 1, 1);
    grid.attach(&create_label("Current now:"), 0, 6, 1, 1);
    grid.attach(&create_label(current_now.as_str()), 1, 6, 1, 1);
    grid.attach(&create_label("Avg current:"), 0, 7, 1, 1);
    grid.attach(&create_label(current_avg.as_str()), 1, 7, 1, 1);
    grid.attach(&create_label("% Remaining:"), 0, 8, 1, 1);
    grid.attach(&create_label(percent.as_str()), 1, 8, 1, 1);
    grid.attach(&create_label("Time remaining:"), 0, 9, 1, 1);
    grid.attach(&create_label(time.as_str()), 1, 9, 1, 1);
    grid.attach(&create_label("Battery health:"), 0, 10, 1, 1);
    grid.attach(&create_label(health.as_str()), 1, 10, 1, 1);
    grid.attach(&create_label("Abs % remaining:"), 0, 11, 1, 1);
    grid.attach(&create_label(abs_percent.as_str()), 1, 11, 1, 1);

    grid.set_row_spacing(6);
    grid.set_column_spacing(6);

    let dialog = MessageDialog::new(None::<&Window>,
                                    DialogFlags::MODAL,
                                    MessageType::Info,
                                    ButtonsType::Ok,
                                    "Battery Stats");

    let content = dialog.get_content_area();
    content.pack_start(&grid, true, true, 6);
    content.set_border_width(18);
    dialog.show_all();
    dialog.run();

    Ok(())
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

/// For the purposes of i3blocks, 3 corresponds to a right click. When
/// that happens, fork() and then show a dialog. If we don't fork(), then
/// the dialog prevents this process from exiting and the block doesn't
/// update. The printing to the console is done in the parent thread.
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
                    if show_gtk_dialog(&bat).is_ok() {
                        std::process::exit(0);
                    } else {
                        std::process::exit(1);
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

    Ok(())
}
