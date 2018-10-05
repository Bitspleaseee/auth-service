use std::io::prelude::*;
use std::fs::OpenOptions;
use std::path::Path;
use std::fs::File;

// Create log for login/register attempt
pub fn create_log(request : &str, username: &str) {

    let file_name = "authLogFile";

    // Check if file exists
    // If it doesn't, create it
    let file_exists : bool = Path::new(file_name).exists();
    if !file_exists {
        if let Err(e) = File::create(file_name) {
            eprintln!("Couldn't create file: {}", e);
            return;
        }
    }

    // Document the time
    let now = chrono::Local::now();
    let time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    // Write to file
    let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(file_name)
                    .unwrap();

    if let Err(e) = writeln!(file, "[{}]: {} from user: {}", time, request, username) {
        eprintln!("Couldn't write to file: {}", e);
    }
}