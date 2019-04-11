use regex::Regex;
use serde_json::Value;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use std::process::{exit, Command, Stdio};
use std::env::args;

fn run_fzf(input: &str) -> String {
    let child = Command::new("fzf")
        .args(&["-f", input, "--no-sort", "--inline-info"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run fzf command");

    let output = child.wait_with_output().expect("Failed to wait fzf command");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn load_config(config_path: &Path) -> Result<Value, Box<dyn Error>> {
    let config_file = File::open(config_path)?;
    let config_reader = BufReader::new(config_file);
    let v: Value = serde_json::from_reader(config_reader)?;
    Ok(v)
}

fn main() -> Result<(), Box<dyn Error>> {
    let current_config_path = Path::new(".fzf_alt.json");
    let alternate_config_path = Path::new("~/.fzf_alt.json");

    let config_path = if current_config_path.exists() {
        current_config_path
    } else if alternate_config_path.exists() {
        alternate_config_path
    } else {
        eprintln!(".fzf_alt.json not found in current dir or home dir");
        exit(1)
    };

    let config = load_config(&config_path)?;
    let args: Vec<String> = args().collect();

    if args.len() == 1 {
        exit(1);
    }

    // Guaranteed to exist, because we check previously if args is empty
    let filename = if let Some(filename) = args.get(1) {
        filename
    } else {
        unreachable!()
    };

    match (args.get(2), args.get(3)) {
        (None, None) => exit(1),
        (Some(filetype), None) => {
            let filetype_config: &Value = config
                .get(filetype)
                .ok_or(format!("{} could not be found in config", filetype))?;

            let strip_regex: &str = filetype_config
                .get("strip")
                .and_then(Value::as_str)
                .ok_or(format!("You must define strip in {}", filetype))?;

            let is_test_regex: &str = filetype_config
                .get("is_test")
                .and_then(Value::as_str)
                .ok_or(format!("You must define is_test in {}", filetype))?;

            let re = Regex::new(strip_regex)?;
            let is_test_re = Regex::new(is_test_regex)?;
            let is_test = |file| {
                is_test_re.is_match(file)
            };

            let current_file_stripped = re
                .captures(filename)
                .and_then(|caps| caps.name("p"))
                .map(|m| m.as_str())
                .unwrap_or(filename);

            let result = run_fzf(current_file_stripped);
            let mut result = result
                .split_whitespace()
                .filter(|line| {
                    is_test(line) ^ is_test(filename)
                });
            let result = result
                .next()
                .unwrap_or_else(|| exit(1));

            println!("{}", result);
        },
        (Some(_filetype), Some(_alternate)) => {}
        _ => unreachable!()
    }

    Ok(())
}

