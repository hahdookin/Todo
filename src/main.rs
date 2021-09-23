//use std::io;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::Path;

use chrono::DateTime;

mod util;
mod entry;
use entry::Entry;

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
    Mod(usize, HashMap<String, String>),
    Del(usize),
    List,
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

    if args.len() == 1 {
        // print usage
        // and return
    }

    let command = match args[0].as_str() {

        "list" => {
            Command::List
        }

        // add "group" "date" "desc"
        "add" => {
            let group = args[1].to_owned();
            let date = args[2].to_owned();
            let desc = args[3].to_owned();
            Command::Add(group, date, desc)
            //Command::Add(args[1].to_owned(), args[2].to_owned(), args[3].to_owned())
        }

        // mod id {param=val}+
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
    let expanded_path = util::expand_tilde() + "/.ctodocache";
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
    

    // Read file into a buffer (and grab byte len)
    let mut s = String::new();
    let nbytes = file.read_to_string(&mut s);

    let lines: Vec<&str> = s.lines().collect();

    // Grab all the entries
    let mut entries: Vec<Entry> = Vec::new();
    for line in lines {
        let entry = Entry::from_entry_line(line);
        entries.push(entry);
    }

    
    let mut just_list = false;
    // Handle commands here!
    match command {
        Command::List => {
            just_list = true;
        }
        Command::Add(group, date, desc) => {
            let highest = entry::highest_entry_id(&entries);
            let mut with_tz = date.to_owned();
            with_tz.push_str(entry::TZ);
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
    if just_list {
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

    // Empty the file
    let mut file = fs::File::create(&path).unwrap();

    // Write new entries to file
    let contents_vec: Vec<String> = entries
        .iter()
        .map(|e| { e.as_file_line() })
        .collect();
    let contents = contents_vec.join("\n");

    file.write_all(contents.as_bytes());
}
