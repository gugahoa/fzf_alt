use regex::Regex;
use serde_json::Value;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use std::process::{exit, Command, Stdio};
use std::env::args;

struct Alternate {
    filename: String,
    is_test_regex: Regex,
    strip_regex: Regex
}

impl Alternate {
    fn new(filetype: String, filename: String, config: Value) -> Alternate {
        let filetype_config: &Value = config
            .get(&filetype)
            .expect(&format!("{} could not be found in config", filetype));

        let strip_regex: &str = filetype_config
            .get("strip")
            .and_then(Value::as_str)
            .expect(&format!("You must define strip in {}", filetype));

        let is_test_regex: &str = filetype_config
            .get("is_test")
            .and_then(Value::as_str)
            .expect(&format!("You must define is_test in {}", filetype));

        let is_test_re = Regex::new(is_test_regex)
            .expect("failed to parse test regex");


        let strip_re = Regex::new(strip_regex)
            .expect("failed to parse strip regex");

        Alternate {
            strip_regex: strip_re,
            is_test_regex: is_test_re,
            filename: filename,
        }
    }

    fn strip_filename(&self) -> &str {
        self.strip_regex
            .captures(&self.filename)
            .and_then(|caps| caps.name("p"))
            .map(|m| m.as_str())
            .unwrap_or(&self.filename)
    }

    fn is_test(&self, filename: &str) -> bool {
        self.is_test_regex.is_match(filename)
    }

    fn get_alternate_file<'a>(&'a self, files: &'a str) -> Option<&'a str> {
        let mut result = files
            .split_whitespace()
            .filter(|file| {
                self.is_test(file) ^ self.is_test(&self.filename)
            });

        result
            .next()
    }
}

fn run_fzf(input: &str) -> String {
    let child = Command::new("fzf")
        .args(&["-f", input, "--no-sort", "--inline-info"])
        .stdout(Stdio::piped())
        .stdin(Stdio::inherit())
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

    if args.len() < 2 {
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
            let alternate = Alternate::new(filetype.to_owned(), filename.to_owned(), config);
            let files = run_fzf(alternate.strip_filename());
            let result = alternate.get_alternate_file(&files);

            println!("{}", result.unwrap_or_else(|| exit(1)));
        },
        (Some(_filetype), Some(_alternate)) => {}
        _ => unreachable!()
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_CASE: &str = "lib/example/content.ex
lib/example/content/question.ex
lib/example/content/module_question.ex
lib/example/content/feedback.ex
lib/example/content/module.ex
lib/example/content/exam.ex
lib/example_web/controllers/newsletter_controller.ex
lib/example_web/controllers/user_controller.ex
lib/example_web/controllers/feedback_controller.ex
lib/example_web/controllers/module_controller.ex
lib/example_web/controllers/question_controller.ex
lib/example_web/controllers/auth_controller.ex
lib/example_web/controllers/page_controller.ex
lib/example_web/controllers/fallback_controller.ex
lib/example_web/controllers/exam_controller.ex
test/example/content/content_test.exs
test/example_web/controllers/module_controller_test.exs
test/example_web/controllers/feedback_controller_test.exs
test/example_web/controllers/exam_controller_test.exs
test/example_web/controllers/question_controller_test.exs
test/example_web/controllers/page_controller_test.exs
test/example_web/controllers/user_controller_test.exs
test/example_web/controllers/auth_controller_test.exs
test/example_web/controllers/newsletter_controller_test.exs";

    const CONFIG_STR: &str = "{
        \"elixir\": {
            \"is_test\": \"_test.exs$\",
            \"strip\": \"(?P<p>[^_\\/]+)_?(\\\\w+)?.ex$\",
            \"view\": \"{}_view.ex\"
        }
    }";

    #[test]
    fn test_elixir_content_alternate() {
        let config = serde_json::from_str(CONFIG_STR)
            .expect("Failed to parse CONFIG_STR");

        let alternate = Alternate::new(
            "elixir".to_owned(),
            "lib/example/content.ex".to_owned(),
            config
            );

        assert_eq!(
            alternate.get_alternate_file(TEST_CASE),
            Some("test/example/content/content_test.exs")
            );
    }

    #[test]
    fn test_elixir_content_test_alternate() {
        let config = serde_json::from_str(CONFIG_STR)
            .expect("Failed to parse CONFIG_STR");

        let alternate = Alternate::new(
            "elixir".to_owned(),
            "test/example/content/content_test.exs".to_owned(),
            config
            );

        assert_eq!(
            alternate.get_alternate_file(TEST_CASE),
            Some("lib/example/content.ex")
            );
    }
}
