use std::env;
use std::time::SystemTime;

pub fn expand_tilde() -> String {
    env::var("HOME").expect("Couldn't grab home var")
}

pub fn time_since_epoch() -> u64 {
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    now.expect("Time error!").as_secs()
}
