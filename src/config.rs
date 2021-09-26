// Format for dates: Sun(%a) Sep(%b) 12(%d) 12:30 PM(%I:%M %p)
// Uses chrono::format rules
pub const TIME_FMT: &str = "%a %b %d %I:%M %p";
// Format for entries:
// %n: id
// %t: date (time_fmt)
// %s: desc
pub const PRINT_FMT: &str = "[%n] Due: %t, %s";

pub struct Config {
    pub time_fmt: String,
    pub print_fmt: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            time_fmt: TIME_FMT.to_owned(),
            print_fmt: PRINT_FMT.to_owned(),
        }
    }
}

