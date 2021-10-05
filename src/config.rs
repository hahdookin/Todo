use termion::color::*;

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

    pub ignore_group_case: bool,

    pub today_color: String,
    pub less_than_day_color: String,
    pub less_than_week_color: String,
    pub greater_than_week_color: String,
    pub past_due_color: String,
    pub group_color: String,
}

impl Config {
    pub fn default() -> Self {
        Self {
            time_fmt: TIME_FMT.to_owned(),
            print_fmt: PRINT_FMT.to_owned(),

            ignore_group_case: false,

            today_color: Blue.fg_str().to_owned(),
            less_than_day_color: Red.fg_str().to_owned(), //String::from("red"),
            less_than_week_color: Yellow.fg_str().to_owned(), //String::from("yellow"),
            greater_than_week_color: LightWhite.fg_str().to_owned(),
            past_due_color: Red.fg_str().to_owned(), //String::from("red"),
            group_color: Green.fg_str().to_owned(),
        }
    }

}

