use std::env;

mod battery;

use battery::Battery;
use battery::BatteryStatus;

fn console_output(bat: Battery) {
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
    println!("{0:}{0:}", out);
    if percent <= 5 {
        std::process::exit(33);
    }
}

fn main() {
    let battery_name = match env::var("BLOCK_INSTANCE") {
        Ok(val) => val,
        Err(_) => "BAT0".to_owned(),
    };
    let bat = Battery::initialize(&battery_name);
    console_output(bat);
}
