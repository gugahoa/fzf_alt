use regex::Regex;
use serde_json::Value;
use std::env::args;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::{exit, Command, Stdio};

struct Alternate {
    filename: String,
    is_test_regex: Regex,
    strip_regex: Regex,
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

        let is_test_re = Regex::new(is_test_regex).expect("failed to parse test regex");

        let strip_re = Regex::new(strip_regex).expect("failed to parse strip regex");

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
            .filter(|file| self.is_test(file) ^ self.is_test(&self.filename));

        result.next()
    }
}

fn run_fzf<I: Into<Stdio>>(input: &str, stdin: Option<I>) -> String {
    let child = Command::new("fzf")
        .args(&["-f", input, "--no-sort", "--inline-info"])
        .stdout(Stdio::piped())
        .stdin(stdin.map(Into::into).unwrap_or(Stdio::inherit()))
        .spawn()
        .expect("Failed to run fzf command");

    let output = child
        .wait_with_output()
        .expect("Failed to wait fzf command");
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
            let files = run_fzf::<Stdio>(alternate.strip_filename(), None);
            let result = alternate.get_alternate_file(&files);

            println!("{}", result.unwrap_or_else(|| exit(1)));
        }
        (Some(_filetype), Some(_alternate)) => {}
        _ => unreachable!(),
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::prelude::*;
    use std::io::SeekFrom;
    use tempfile::tempfile;

    const TEST_CASE: &str = "lib/example.ex
lib/example_web.ex
test/test_helper.exs
lib/example/repo.ex
lib/example/account.ex
lib/example/content.ex
lib/example/marketing.ex
lib/example/application.ex
lib/guardian/pipeline.ex
lib/guardian/error_handler.ex
lib/guardian/guardian.ex
lib/example_web/gettext.ex
lib/example_web/router.ex
lib/example_web/endpoint.ex
test/support/test_helpers.ex
test/support/conn_case.ex
test/support/data_case.ex
test/support/channel_case.ex
lib/example/content/question.ex
lib/example/content/module_question.ex
lib/example/content/feedback.ex
lib/example/content/module.ex
lib/example/content/exam.ex
lib/example/marketing/newsletter.ex
lib/example/account/user.ex
lib/example_web/controllers/newsletter_controller.ex
lib/example_web/controllers/user_controller.ex
lib/example_web/controllers/feedback_controller.ex
lib/example_web/controllers/module_controller.ex
lib/example_web/controllers/question_controller.ex
lib/example_web/controllers/auth_controller.ex
lib/example_web/controllers/page_controller.ex
lib/example_web/controllers/fallback_controller.ex
lib/example_web/controllers/exam_controller.ex
lib/example_web/channels/user_socket.ex
lib/example_web/views/error_view.ex
lib/example_web/views/layout_view.ex
lib/example_web/views/error_helpers.ex
lib/example_web/views/page_view.ex
lib/example_web/views/question_view.ex
lib/example_web/views/exam_view.ex
lib/example_web/views/auth_view.ex
lib/example_web/views/feedback_view.ex
lib/example_web/views/user_view.ex
lib/example_web/views/newsletter_view.ex
lib/example_web/views/changeset_view.ex
lib/example_web/views/module_view.ex
test/example/marketing/marketing_test.exs
test/example/account/account_test.exs
test/example/content/content_test.exs
test/example_web/views/error_view_test.exs
test/example_web/views/layout_view_test.exs
test/example_web/views/page_view_test.exs
test/example_web/controllers/module_controller_test.exs
test/example_web/controllers/feedback_controller_test.exs
test/example_web/controllers/exam_controller_test.exs
test/example_web/controllers/question_controller_test.exs
test/example_web/controllers/page_controller_test.exs
test/example_web/controllers/user_controller_test.exs
test/example_web/controllers/auth_controller_test.exs
lib/example_web/templates/layout/app.html.eex
test/example_web/controllers/newsletter_controller_test.exs
lib/example_web/templates/page/index.html.eex
";

    const CONFIG_STR: &str = "{
        \"elixir\": {
            \"is_test\": \"_test.exs$\",
            \"strip\": \"(?P<p>[^_\\/]+)_?(\\\\w+)?.exs?$\"
        }
    }";

    fn test_case_fixture(input: &str) -> String {
        let mut tmp_file = tempfile().expect("Failed to create tmp file for test");
        write!(&mut tmp_file, "{}", TEST_CASE).expect("Failed to write to tmp file");
        tmp_file.seek(SeekFrom::Start(0)).unwrap();

        run_fzf(input, Some(tmp_file))
    }

    #[test]
    fn test_elixir_content_alternate() {
        let config = serde_json::from_str(CONFIG_STR).expect("Failed to parse CONFIG_STR");

        let alternate = Alternate::new(
            "elixir".to_owned(),
            "lib/example/content.ex".to_owned(),
            config,
        );

        let test_case = test_case_fixture(alternate.strip_filename());

        assert_eq!(
            alternate.get_alternate_file(&test_case),
            Some("test/example/content/content_test.exs")
        );
    }

    #[test]
    fn test_elixir_content_test_alternate() {
        let config = serde_json::from_str(CONFIG_STR).expect("Failed to parse CONFIG_STR");

        let alternate = Alternate::new(
            "elixir".to_owned(),
            "test/example/content/content_test.exs".to_owned(),
            config,
        );

        let test_case = test_case_fixture(alternate.strip_filename());

        assert_eq!(
            alternate.get_alternate_file(&test_case),
            Some("lib/example/content.ex")
        );
    }
}
