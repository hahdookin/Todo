use std::collections::HashMap;
use std::fmt;

use chrono::format;
use chrono::{offset, DateTime, Local, NaiveDateTime, TimeZone, Utc};

// Format for dates: Sun(%a) Sep(%b) 12(%d) 12:30 PM(%I:%M %p)
pub const TIME_FMT: &str = "%a %b %d %I:%M %p";
// TODO: Please find a solution for this hack
pub const TZ: &str = "-0400";

#[derive(Debug)]
pub struct Entry {
    pub id: usize,
    pub group: String,
    pub date: isize,
    pub desc: String,
}

impl Entry {

    pub fn from_entry_line(line: &str) -> Self {
        let items: Vec<&str> = line.split(',').collect();

        let id = items[0].parse::<usize>().unwrap();
        let group = items[1].to_owned();
        let date = items[2].parse::<isize>().unwrap();
        let desc = items[3].to_owned();

        Entry {
            id: id,
            group: group,
            date: date,
            desc: desc,
        }
    }

    pub fn from_elements(id: usize, group: &String, date: isize, desc: &String) -> Self {
        Entry {
            id: id,
            group: group.to_owned(),
            date: date,
            desc: desc.to_owned(),
        }
    }

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

    //1,cs288,1631684966,this is the 1 desc
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
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let l: DateTime<Local> = Local.timestamp(self.date as i64, 0);
        let t = l.format(TIME_FMT);
        write!(f, "[{}] Due: {}, {}", self.id, t, self.desc)
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
