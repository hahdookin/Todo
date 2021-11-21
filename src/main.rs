//use std::io;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::Path;

use chrono::DateTime;
use termion::color;

mod config;
mod entry;
mod util;

use entry::Entry;

const KEYWORDS: [&str; 3] = ["group", "desc", "date"];

// Takes command line args in form of key=val
// and returns a HashMap of {key: val}
fn parse_mod_args(args: &Vec<String>) -> HashMap<String, String> {
    let mut res: HashMap<String, String> = HashMap::new();

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
            x.push_str(entry::TZ);
            DateTime::parse_from_str(&x, "%m/%d/%Y %I:%M %P%z").unwrap();
        }

        res.insert(key, val);
    }

    res
}

#[derive(Debug)]
enum Command {
    Add(String, String, String),
    //Add{ group: String, date_str: String, desc: String },
    Mod(usize, HashMap<String, String>),
    Del(usize),
    List,
    Reindex,
    Unknown,
}

fn usage() {
    println!("USAGE:");
    println!("    todo <command> [<args>]");
    println!("");
    println!("OPTIONS:");
    println!("  list:");
    println!("    -s    Sort entries by due date");
    println!("");
    println!("COMMANDS:");
    println!("    l[ist], ls    List entries");
    println!("    a[dd]         Add an entry");
    println!("    m[od]         Modify an entry");
    println!("    d[el], rm     Remove an entry");
    println!("    reindex       Reindex existing entry IDs");
    println!("");

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
    let args: Vec<String> = env::args().skip(1).filter(|arg| arg.chars().nth(0).unwrap() != '-').collect();

    // No commands, print usage
    if args.len() == 0 {
        usage();
        return;
    }

    // Options
    let mut sort = false;
    for arg in env::args().skip(1) {
        if arg.chars().nth(0).unwrap() == '-' {
            match arg.chars().nth(1).unwrap() {
                's' => {
                    sort = true;
                },
                _ => {}
            }
        }
    }

    // Determine the command
    let command = match args[0].as_str() {

        // l[ist]|ls
        "list" | "lis" | "li" | "l" | "ls" => {
            Command::List
        }

        // a[dd] "group" "date" "desc"
        "add" | "ad" | "a" => {
            let group = args[1].to_owned();
            let date = args[2].to_owned();
            let desc = args[3].to_owned();
            Command::Add(group, date, desc)
        }

        // m[od] id {param=val}+
        "mod" | "mo" | "m" => {
            if let Ok(id) = args[1].parse::<usize>() {
                Command::Mod(id, parse_mod_args(&args))
            } else {
                panic!("Invalid id: {}", args[1]);
            }
        }

        // d[el]|rm id
        "del" | "de" | "d" | "rm" => {
            if args.len() < 2 {
                panic!("No id given for del");
            }
            if let Ok(id) = args[1].parse::<usize>() {
                Command::Del(id)
            } else {
                panic!("Invalid id: {}", args[1]);
            }
        }

        "reindex" => {
            Command::Reindex
        }

        _ => Command::Unknown,
    };

    // Grab the path
    let expanded_path = util::expand_tilde() + "/.todocache";
    let path = Path::new(&expanded_path);
    let display = path.display();

    // Open the file
    let mut file = match fs::File::open(&path) {
        Err(why) => {
            // TODO: make the file!
            panic!("Couldn't open {}: {}", display, why)
        }
        Ok(file) => file,
    };

    // Configuration
    let mut cfg = config::Config::default();
    cfg.ignore_group_case = true; // TODO: Implement this setting

    // Read file into a buffer (and grab byte len)
    let mut s = String::new();
    let _nbytes = file.read_to_string(&mut s);

    let lines: Vec<&str> = s.lines().collect();

    // Grab all the entries
    let mut entries: Vec<Entry> = Vec::new();
    for line in lines {
        let entry = Entry::from_entry_line(line, &cfg);
        entries.push(entry);
    }
    
    // Handle commands here!
    let mut just_list = false;
    let mut should_reindex = false;

    match command {

        Command::Add(group, date, desc) => {
            let highest = entry::highest_entry_id(&entries);
            let mut with_tz = date.to_owned();
            with_tz.push_str(entry::TZ);
            let res = Entry::from_elements(
                    highest + 1, 
                    group, 
                    DateTime::parse_from_str(&with_tz, "%m/%d/%Y %I:%M %P%z").unwrap().timestamp() as isize,
                    desc
                );
            print!("Adding entry: ");
            res.print(&cfg);
            entries.push(res);
        }

        Command::Mod(id, newvals) => {
            let id_index = entries
                .iter()
                .position(|e| e.id == id)
                .expect("No entry with index"); // Error handle here
   
            print!("Updating entry: ");
            entries[id_index].print(&cfg);
            entries[id_index].update_values(&newvals);
            print!("Updated: ");
            entries[id_index].print(&cfg);
        }

        Command::Del(id) => {
            let id_index = entries
                .iter()
                .position(|e| e.id == id)
                .expect("No entry with index"); // Error handle here

            print!("Deleting entry: ");
            entries[id_index].print(&cfg);
            entries.remove(id_index);
        }

        Command::List => {
            just_list = true;
        }

        Command::Reindex => {
            should_reindex = true;
        }

        Command::Unknown => {
            usage();
            return;
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
    
    // Get longest group name
    let max_group_name_len = groups.iter().map(|g| g.chars().count() ).max().unwrap();

    // Reindex the entries starting from 0
    if should_reindex {
        let mut i = 0;
        for entry in entries.iter_mut() {
            entry.id = i;
            i += 1;
        }
    }

    // Print all entries for each group
    if just_list {
        use chrono::{Local, TimeZone};
        let now = Local::now().timestamp() as i64; // Now in seconds from epoch
        let l: DateTime<Local> = Local.timestamp(now, 0); // Now as DateTime
        let t = l.format(cfg.time_fmt.as_str()); // Now as Str rep

        print!("Today is: ");
        print!("{}", color::LightCyan.fg_str());
        println!("{}", t);
        print!("{}", color::LightWhite.fg_str());

        if sort {
            // Sorted print
            entries.sort_by(|a, b| {
                (now - b.date as i64).cmp(&(now - a.date as i64))
            });
            for e in entries {
                print!("{}", cfg.group_color);
                print!("{}: ", e.group);
                print!("{}\n", color::LightWhite.fg_str());
                e.print(&cfg);
            }
        } else {
            // Regular print
            for group in groups.iter() {
                print!("{}", cfg.group_color);
                println!("{}:", group);
                print!("{}", color::LightWhite.fg_str());
                for entry in entries.iter() {
                    if *group == entry.group {
                        entry.print(&cfg);
                    }
                }
            }
        }
        return;
    }

    // Empty the file
    let mut file = fs::File::create(&path).unwrap();

    // Write new entries to file
    let contents: Vec<String> = entries
        .iter()
        .map(|e| { e.as_file_line() })
        .collect();
    let contents = contents.join("\n");

    file.write_all(contents.as_bytes());
}
