use confy;
use fzf_alt::config::AppConfig;
use regex::Regex;
use std::env::args;
use std::error::Error;
use std::process::{exit, Command, Stdio};

struct Alternate {
    filename: String,
    is_test_regex: Regex,
    strip_regex: Regex,
}

impl Alternate {
    fn new(filetype: String, filename: String) -> Option<Alternate> {
        let cfg: AppConfig = confy::load("fzf_alt").expect("Failed to load fzf_alt config");

        let filetype_cfg = cfg.get_filetype_config(&filetype)?;

        Some(Alternate {
            strip_regex: filetype_cfg.strip.to_owned(),
            is_test_regex: filetype_cfg.is_test.to_owned(),
            filename: filename,
        })
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

fn run_fzf(input: &str, stdin: impl Into<Stdio>) -> String {
    let child = Command::new("fzf")
        .args(&["-f", input, "--no-sort", "--inline-info"])
        .stdout(Stdio::piped())
        .stdin(stdin)
        .spawn()
        .expect("Failed to run fzf command");

    let output = child
        .wait_with_output()
        .expect("Failed to wait fzf command");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = args().collect();

    if args.len() < 2 {
        eprintln!("too few args provided");
        exit(1);
    }

    // Guaranteed to exist, because we check previously if args is empty
    let filename = if let Some(filename) = args.get(1) {
        filename
    } else {
        eprintln!("filename should be provided");
        exit(1);
    };

    match (args.get(2), args.get(3)) {
        (None, None) => {
            eprintln!("filetype should be provided");
            exit(1);
        }
        (Some(filetype), None) => {
            let alternate = Alternate::new(filetype.to_owned(), filename.to_owned());
            let alternate = if let Some(alt) = alternate {
                alt
            } else {
                eprintln!("{} not found in fzf_alt config", filetype);
                exit(1)
            };

            let files = run_fzf(alternate.strip_filename(), Stdio::inherit());
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

    fn test_case_fixture(input: &str) -> String {
        let mut tmp_file = tempfile().expect("Failed to create tmp file for test");
        write!(&mut tmp_file, "{}", TEST_CASE).expect("Failed to write to tmp file");
        tmp_file.seek(SeekFrom::Start(0)).unwrap();

        run_fzf(input, tmp_file)
    }

    #[test]
    fn test_elixir_content_alternate() {
        let alternate = Alternate::new("elixir".to_owned(), "lib/example/content.ex".to_owned())
            .expect("elixir not found in fzf_alt config");

        let test_case = test_case_fixture(alternate.strip_filename());

        assert_eq!(
            alternate.get_alternate_file(&test_case),
            Some("test/example/content/content_test.exs")
        );
    }

    #[test]
    fn test_elixir_content_test_alternate() {
        let alternate = Alternate::new(
            "elixir".to_owned(),
            "test/example/content/content_test.exs".to_owned(),
        )
        .expect("elixir not found in fzf_alt config");

        let test_case = test_case_fixture(alternate.strip_filename());

        assert_eq!(
            alternate.get_alternate_file(&test_case),
            Some("lib/example/content.ex")
        );
    }
}
