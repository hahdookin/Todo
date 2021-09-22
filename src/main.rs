//use std::io;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::SystemTime;

use chrono::format;
use chrono::{offset, DateTime, Local, NaiveDateTime, TimeZone, Utc};

#[derive(Debug)]
struct Entry {
    id: usize,
    group: String,
    date: isize,
    desc: String,
}

impl Entry {
    fn from_entry_line(line: &str) -> Self {
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
    fn from_elements(id: usize, group: &String, date: isize, desc: &String) -> Self {
        Entry {
            id: id,
            group: group.to_owned(),
            date: date,
            desc: desc.to_owned(),
        }
    }
    fn update_values(&mut self, keyvals: &HashMap<String, String>) {
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
}

// Format for dates: Sun(%a) Sep(%b) 12(%d) 12:30 PM(%I:%M %p)
const TIME_FMT: &str = "%a %b %d %I:%M %p";
// TODO: Please find a solution for this hack
const TZ: &str = "-0400";

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let l: DateTime<Local> = Local.timestamp(self.date as i64, 0);
        let t = l.format(TIME_FMT);
        write!(f, "[{}] Due: {}, {}", self.id, t, self.desc)
    }
}

fn expand_tilde() -> String {
    env::var("HOME").expect("Couldn't grab home var")
}

fn time_since_epoch() -> u64 {
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    now.expect("Time error!").as_secs()
}

const KEYWORDS: [&str; 3] = ["group", "desc", "date"];

// Takes command line args in form of key=val
// and returns a HashMap of {key: val}
fn parse_mod_args(args: &Vec<String>) -> HashMap<String, String> {
    let mut res: HashMap<String, String> = HashMap::new();

    let input_to_key_val = |item: &String| {
        let x: Vec<&str> = item.split('=').collect();
        (x[0].to_owned(), x[1].to_owned())
    };

    // skip 'mod' and 'id'
    for item in args.into_iter().skip(2) {
        // Make sure only one '='
        let equals_count = item.chars().filter(|c| *c == '=').count();
        if equals_count != 1 {
            panic!("Expected 1 '=', found {} in {}", equals_count, item);
        }

        // Make sure '=' not first or last char
        let equals_index = item.find('=').unwrap();
        let last_index = item.chars().count() - 1;
        if equals_index == 0 || equals_index == last_index {
            panic!("bad '=' placement: {}", item);
        }

        let split: Vec<&str> = item.split('=').collect();
        let (key, val) = (split[0].to_owned(), split[1].to_owned());

        // Verify key
        if !KEYWORDS.contains(&key.as_str()) {
            panic!("Not a valid keyword: \"{}\"", key);
        }
        // TODO: Verify all vals
        if res.contains_key("date") {
            // Verify that date string can be converted to DateTime
            let mut x = res["date"].to_owned();
            x.push_str(TZ);
            DateTime::parse_from_str(&x, "%m/%d/%Y %I:%M %P%z").unwrap();
        }

        res.insert(key, val);
    }

    res
}

fn highest_entry_id(entries: &Vec<Entry>) -> usize {
    let mut highest: usize = 0;
    for entry in entries.iter() {
        if entry.id > highest {
            highest = entry.id;
        }
    }
    highest
}

#[derive(Debug)]
enum Command {
    Add(String, String, String),
    Mod(usize, HashMap<String, String>),
    Del(usize),
    Unknown,
}

/*
 * Format of .todocache:
 * Line 1: <id>,<group>,<due date>,<desc>\n
 * ...
 * Line n: <id>,<group>,<due date>,<desc>
 */

/*
 * Ways to call:
 * todo [options]
 * todo add <entry>
 * todo mod <entryID> {<param>=<val>}+
 * todo del <entryID>
 *
 * <entry> := <group> <due date> <desc>
 * <param> := <group> | <due date> | <desc>
 * <due date> := <ms since epoch>
 */
// Format for dates: Sun(%a) Sep(%b) 12(%d) 12:30 PM(%I:%M %p)
//%a %b %d %I:%M %p
fn main() {
    // Handle args (skip program path)
    let args: Vec<String> = env::args().skip(1).collect();

    let command = match args[0].as_str() {
        "add" => {
            Command::Add(args[1].to_owned(), args[2].to_owned(), args[3].to_owned())
        },
        // mod id param=val
        "mod" => {
            if let Ok(id) = args[1].parse::<usize>() {
                Command::Mod(id, parse_mod_args(&args))
            } else {
                panic!("Invalid id: {}", args[1]);
            }
        }
        // del id
        "del" => {
            if args.len() < 2 {
                panic!("No id given for del");
            }
            if let Ok(id) = args[1].parse::<usize>() {
                Command::Del(id)
            } else {
                panic!("Invalid id: {}", args[1]);
            }
        }

        _ => Command::Unknown,
    };

    // Grab the path
    let expanded_path = expand_tilde() + "/.ctodocache";
    let path = Path::new(&expanded_path);
    let display = path.display();

    // Open the file
    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read file into a buffer (and grab byte len)
    let mut s = String::new();
    let nbytes = file.read_to_string(&mut s);

    let mut lines: Vec<&str> = s.lines().collect();

    // Grab all the entries
    let mut entries: Vec<Entry> = Vec::new();
    for line in lines {
        let entry = Entry::from_entry_line(line);
        entries.push(entry);
    }

    

    // Handle commands here!
    match command {
        Command::Add(group, date, desc) => {
            let highest = highest_entry_id(&entries);
            let mut with_tz = date.to_owned();
            with_tz.push_str(TZ);
            let res = Entry::from_elements(
                    highest + 1, 
                    &group, 
                    DateTime::parse_from_str(&with_tz, "%m/%d/%Y %I:%M %P%z").unwrap().timestamp() as isize,
                    &desc
                );
            entries.push(res);
        }
        Command::Mod(id, newvals) => {
            let (id_index, _) = entries
                .iter()
                .enumerate()
                .find(|(i, e)| e.id == id)
                .unwrap();
                
            entries[id_index].update_values(&newvals);
        }
        Command::Del(id) => {
            let (id_index, _) = entries
                .iter()
                .enumerate()
                .find(|(i, e)| e.id == id)
                .unwrap(); // Error handle here

            println!("Deleting entry: {}", &entries[id_index]);
            entries.remove(id_index);
        }
        Command::Unknown => {
            // Print usage
        }
    };

    // Extract unique groups
    let mut groups: Vec<String> = Vec::new();
    for entry in entries.iter() {
        let cur_group = &entry.group;
        if !groups.iter().any(|g| g == cur_group) {
            groups.push(cur_group.to_owned());
        }
    }

    // Print all entries for each group
    for group in groups.iter() {
        println!("{}:", group);
        for entry in entries.iter() {
            if *group == entry.group {
                println!("{}", entry);
            }
        }
        println!("");
    }
}
