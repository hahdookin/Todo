use std::fmt;
use std::collections::HashMap;

use chrono::format;
use chrono::{offset, DateTime, Local, NaiveDateTime, TimeZone, Utc};

use crate::config::Config;
use termion::color;

// TODO: Please find a solution for this hack
pub const TZ: &str = "-0400";

#[derive(Debug)]
pub struct Entry {
    pub id: usize,
    pub group: String,
    pub date: isize,
    pub desc: String,
    //pub cfg: &config::Config,
}

impl Entry {

    // Construct an entry from a line stored in .todocache
    // TODO: line.split(',') will not work if Group or Desc have a ',' in them
    pub fn from_entry_line(line: &str, cfg: &Config) -> Self {
        let items: Vec<&str> = line.split(',').collect();

        let id = items[0].parse::<usize>().unwrap();
        let mut group = items[1].to_owned();
        if cfg.ignore_group_case {
            group.make_ascii_lowercase();
        }
        let date = items[2].parse::<isize>().unwrap();
        let desc = items[3].to_owned();

        Self {
            id: id,
            group: group,
            date: date,
            desc: desc,
        }
    }

    // Construct an entry from 'add' command, uses highest available ID as ID
    // e.g: todo add "group" "9/21/2021 11:59 pm" "description goes here"
    pub fn from_elements(id: usize, group: String, date: isize, desc: String) -> Self {
        Self {
            id: id,
            group: group,
            date: date,
            desc: desc,
        }
    }

    // Update the values of a 'mod' command
    // where `keyvals` is a map of valid entry keys
    // to valid entry values
    pub fn update_values(&mut self, keyvals: &HashMap<String, String>) {
        for (k, v) in keyvals.iter() {
            match k.as_str() {
                "id" => self.id = v.parse::<usize>().unwrap(),
                "group" => self.group = v.to_owned(),
                "date" => { 
                    let mut x = v.to_owned();
                    x.push_str(TZ);
                    self.date = DateTime::parse_from_str(&x, "%m/%d/%Y %I:%M %P%z").unwrap().timestamp() as isize;
                },
                "desc" => self.desc = v.to_owned(),

                _ => {},
            }
        }
    }

    // Return the ~/.todocache line representation of the entry
    // 1,cs288,1631684966,this is the 1 desc
    pub fn as_file_line(&self) -> String {
        let mut res = String::new();

        res.push_str(&self.id.to_string());
        res.push(',');

        res.push_str(&self.group);
        res.push(',');

        res.push_str(&self.date.to_string());
        res.push(',');

        res.push_str(&self.desc);

        res
    }

    // %n: id
    // %t: date
    // %s: desc
    pub fn print(&self, cfg: &Config) {
        let l: DateTime<Local> = Local.timestamp(self.date as i64, 0);
        let t = l.format(cfg.time_fmt.as_str());

        // Parse cfg's print_fmt
        let mut in_specifier = false; // ch was '%', next is specifier
        //let mut in_escape = false;
        for ch in cfg.print_fmt.chars() {
            if in_specifier {
                match ch {
                    'n' => {
                        print!("{}", self.id);
                    },
                    't' => {
                        print!("{}", self.get_date_color(cfg));
                        print!("{}", t);
                        print!("{}", color::LightWhite.fg_str());
                    },
                    's' => {
                        print!("{}", self.desc);
                    },
                    _ => {
                        panic!("Bad print_fmt: {}", cfg.print_fmt);
                    },
                }
                in_specifier = false;
            } else {
                if ch == '%' {
                    in_specifier = true;
                } else {
                    print!("{}", ch);
                }
            }
        }
        println!("")
    }

    // Returns the appropriate color to highlight the date
    // according to the time difference
    fn get_date_color<'a>(&self, cfg: &'a Config) -> &'a str {
        let now = Local::now().timestamp() as isize;
        let week_as_secs = 604_800;
        let day_as_secs = 86_400;
        //println!("[{}], [{}], [{}]", self.date, now, self.date - now);
        match self.date - now {
            // Entry's date is less than 1 day away
            x if x < day_as_secs => {
                cfg.less_than_day_color.as_str()
            }
            // Entry's date is less than 1 week away
            x if x < week_as_secs => {
                cfg.less_than_week_color.as_str()
            }
            // Entry is pass due
            x if x < 0 => {
                cfg.past_due_color.as_str()
            }
            // Entry is over a week away
            _ => {
                cfg.greater_than_week_color.as_str()
            }
        }
    }
}


pub fn highest_entry_id(entries: &Vec<Entry>) -> usize {
    let mut highest: usize = 0;
    for entry in entries.iter() {
        if entry.id > highest {
            highest = entry.id;
        }
    }
    highest
}


