use super::runs::check_source;
use super::{gha_kind, RULE_ID};
use crate::codebase::rules::RuleFinding;
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};

const WORKFLOW: &str = ".github/workflows/ci.yml";
const ACTION: &str = ".github/actions/local/action.yml";

fn check(rel: &str, source: &str) -> Vec<RuleFinding> {
    check_source(rel, source)
}

fn step_of(message: &str) -> usize {
    message
        .split("step ")
        .nth(1)
        .and_then(|rest| rest.split([' ', '`']).next())
        .and_then(|step| step.parse().ok())
        .unwrap_or(0)
}

fn spec_of(message: &str) -> String {
    message
        .split(" in `")
        .nth(1)
        .and_then(|rest| rest.split('`').next())
        .unwrap_or("")
        .to_string()
}

fn nth_line(text: &str, line: usize) -> &str {
    text.lines().nth(line - 1).expect("line")
}

fn identity(finding: &RuleFinding) -> String {
    let file = Path::new(&finding.file)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    format!(
        "{file}:{}:{}:{}",
        finding.target.as_deref().unwrap_or(""),
        step_of(&finding.message),
        spec_of(&finding.message)
    )
}

fn fixture_root(scenario: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-docker-host-ports/fixture")
            .join(scenario),
    )
}

fn fixture_files(scenario: &str) -> &'static [&'static str] {
    match scenario {
        "fail" => &[
            ".github/actions/db/action.yml",
            ".github/workflows/ci.yml",
            ".github/workflows/compact.yml",
            ".no-mistakes.yml",
        ],
        "pass" => &[
            ".github/actions/db/action.yml",
            ".github/actions/node/action.yml",
            ".github/workflows/ci.yml",
            ".no-mistakes.yml",
        ],
        "abstain" => &[
            ".github/actions/node/action.yaml",
            ".github/workflows/broken.yml",
            ".github/workflows/ci.yml",
            ".github/workflows/nested/old.yml",
            ".no-mistakes.yml",
        ],
        "flow" => &[".github/workflows/ci.yml", ".no-mistakes.yml"],
        "suppression" => &[
            ".github/workflows/ci.yml",
            ".github/workflows/compact.yml",
            ".github/workflows/disabled.yml",
            ".no-mistakes.yml",
        ],
        other => panic!("unknown fixture scenario {other}"),
    }
}

fn load_fixture(scenario: &str) -> (PathBuf, NoMistakesConfig, Vec<PathBuf>) {
    let root = fixture_root(scenario);
    let config =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let files = fixture_files(scenario)
        .iter()
        .map(|name| root.join(name))
        .collect();
    (root, config, files)
}

fn enabled_config() -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    });
    config
}

#[test]
fn kinds_and_missing_sources_are_skipped() {
    assert_eq!(gha_kind(".github/workflows/ci.yml"), Some(false));
    assert!(gha_kind(".github/workflows/nested/ci.yaml").is_none());
    assert_eq!(gha_kind(ACTION), Some(true));
    assert_eq!(gha_kind(".github/actions/node/action.yaml"), Some(true));
    assert!(gha_kind(".github/workflows/ci.json").is_none());
    assert!(gha_kind(".github/actions/build/ci.yml").is_none());
    assert!(gha_kind("docs/ci.yml").is_none());
    assert!(check("README.md", "run: docker run -p 5432:5432 postgres\n").is_empty());
    assert!(check(WORKFLOW, "jobs: [\n").is_empty());
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let root = Path::new("/missing");
    let findings = super::super::github_actions_workflow_files::scan_files(
        root,
        &[PathBuf::from("README.md"), PathBuf::from(WORKFLOW)],
        &sources,
        &|_, _| panic!("a missing source is not judged"),
    )
    .unwrap();
    assert!(findings.is_empty());
}

#[test]
fn omitted_options_run_and_unknown_options_are_rejected() {
    assert!(
        super::check_with_files(Path::new("."), &enabled_config(), &[])
            .unwrap()
            .is_empty()
    );
    let mut config = enabled_config();
    config.rules[0].options = serde_yaml::from_str("nope: true").unwrap();
    let error = super::check_with_files(Path::new("."), &config, &[]).unwrap_err();
    assert!(error.to_string().contains("invalid options"), "{error}");
}

#[test]
fn dynamic_or_unparsed_commands_stay_silent() {
    let silent = [
        "on: push\njobs:\n  build:\n    steps:\n      - run: docker run -p $(echo 1) img\n",
        "on: push\njobs:\n  build:\n    steps:\n      - run: docker run --mystery 1 -p 5432:5432 postgres\n",
        "on: push\njobs:\n  build:\n    steps:\n      - run: docker run postgres -p 5432:5432\n",
        "on: push\njobs:\n  build:\n    steps:\n      - run: echo docker run -p 5432:5432\n",
        "name: ci\n",
        "on: push\njobs: []\n",
    ];
    for source in silent {
        assert!(check(WORKFLOW, source).is_empty(), "{source}");
    }
    let later = "\
on: push
jobs:
  build:
    steps:
      - run: docker run -p $(echo 1) img && docker run -p 5432:5432 postgres
";
    let findings = check(WORKFLOW, later);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("step 1"));
    assert!(spec_of(&findings[0].message) == "5432:5432");
}

#[test]
fn literal_mappings_name_the_host_port_and_the_fix() {
    let source = "\
on: push
jobs:
  build:
    steps:
      - run: docker run -p 5432:5432 -p 127.0.0.1::80 postgres
";
    let findings = check(WORKFLOW, source);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("pins host port `5432`"));
    assert!(findings[0]
        .message
        .contains("Hosted and shared runners collide"));
    assert!(findings[0].message.contains("127.0.0.1::5432"));
    assert!(findings[0].message.contains("$PORT:5432"));
    assert_eq!(findings[0].rule, RULE_ID);
    assert_eq!(findings[0].target.as_deref(), Some("build"));
}

#[test]
fn compact_same_column_steps_use_the_step_line() {
    let source = "\
on: push
jobs:
  a:
    runs-on: ubuntu-latest
    steps:
    - run: docker run -p 5432:5432 postgres
";
    let findings = check(WORKFLOW, source);
    assert_eq!(findings.len(), 1, "{findings:?}");
    let line = source.lines().nth(findings[0].line - 1).unwrap();
    assert!(line.starts_with("    - run:"), "{line}");
    assert!(line.contains("5432:5432"), "{line}");

    let crlf = source.replace('\n', "\r\n");
    let crlf_findings = check(WORKFLOW, &crlf);
    assert_eq!(crlf_findings.len(), 1, "{crlf_findings:?}");
    assert_eq!(crlf_findings[0].line, findings[0].line);

    let bom = format!("\u{feff}{source}");
    assert_eq!(check(WORKFLOW, &bom).len(), 1);
}

#[test]
fn saved_fixtures_flag_only_literal_host_ports() {
    let (root, config, files) = load_fixture("pass");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");

    let (root, config, files) = load_fixture("abstain");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert!(findings.is_empty(), "{findings:?}");

    let (root, config, files) = load_fixture("fail");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    let mut got: Vec<String> = findings.iter().map(identity).collect();
    got.sort();
    let mut expected = Vec::new();
    for (job, spec) in [
        ("after-heredoc", "5432:5432"),
        ("attached", "5432:5432"),
        ("block", "5432:5432"),
        ("cap-add-after", "5432:5432"),
        ("cap-add-before", "5432:5432"),
        ("captured", "5432:5432"),
        ("chain", "5432:5432"),
        ("container-run", "5432:5432"),
        ("clustered", "5432:5432"),
        ("container-var", "5432:$PORT"),
        ("continued", "5432:5432"),
        ("env-prefix", "5432:5432"),
        ("if-guard", "5432:5432"),
        ("equals", "8080:80"),
        ("ip", "127.0.0.1:5432:5432"),
        ("ipv6", "[::1]:5432:5432"),
        ("known-flags", "127.0.0.1:5432:5432"),
        ("literal", "5432:5432"),
        ("long", "8080:80"),
        ("path", "5432:5432"),
        ("quoted", "5432:5432"),
        ("range", "8000-8002:8000-8002"),
        ("reservation", "5432:5432"),
        ("subshell", "5432:5432"),
        ("subst", "5432:5432"),
        ("subst-value", "5432:5432"),
        ("sudo", "5432:5432"),
        ("two", "5432:5432"),
        ("two", "6379:6379"),
        ("udp", "5432:5432/udp"),
    ] {
        expected.push(format!("ci.yml:{job}:1:{spec}"));
    }
    expected.push("ci.yml:second-step:2:5432:5432".to_string());
    expected.push("compact.yml:a:1:5432:5432".to_string());
    expected.push("compact.yml:b:2:127.0.0.1:5432:5432".to_string());
    expected.push("action.yml:composite:1:5432:5432".to_string());
    expected.sort();
    assert_eq!(got, expected, "{findings:?}");

    let text = |file: &str| std::fs::read_to_string(root.join(file)).unwrap();
    let located = |file: &str, target: &str, step: usize| {
        findings
            .iter()
            .find(|finding| {
                finding.file.ends_with(file)
                    && finding.target.as_deref() == Some(target)
                    && step_of(&finding.message) == step
            })
            .unwrap()
            .line
    };
    let compact = text(".github/workflows/compact.yml");
    assert!(nth_line(&compact, located("compact.yml", "a", 1)).starts_with("    - run:"));
    assert!(nth_line(&compact, located("compact.yml", "b", 2)).contains("--publish"));
    let ci = text(".github/workflows/ci.yml");
    assert!(nth_line(&ci, located("ci.yml", "block", 1)).contains("docker run -p 5432:5432"));
    assert!(nth_line(&ci, located("ci.yml", "continued", 1))
        .trim()
        .starts_with("5432:5432"));
    assert!(nth_line(&ci, located("ci.yml", "literal", 1)).contains("- run:"));
    for finding in &findings {
        assert!(finding.message.contains("Publish a Docker-allocated port"));
    }

    let (root, config, files) = load_fixture("flow");
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].target.as_deref(), Some("build"));
    assert!(spec_of(&findings[0].message) == "5432:5432");
}

#[test]
fn suppression_directives_hide_file_line_and_next_line() {
    let (root, config, files) = load_fixture("suppression");
    let raw = super::check_with_files(&root, &config, &files).unwrap();
    assert_eq!(raw.len(), 6, "{raw:?}");
    let findings = crate::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    let ci = findings
        .iter()
        .find(|finding| finding.file.ends_with("ci.yml"))
        .unwrap();
    assert!(ci.message.contains("step 3"), "{ci:?}");
    assert!(spec_of(&ci.message) == "6379:6379");
    let compact = findings
        .iter()
        .find(|finding| finding.file.ends_with("compact.yml"))
        .unwrap();
    assert_eq!(compact.target.as_deref(), Some("b"));
    assert!(compact.message.contains("step 2"), "{compact:?}");
    let text = std::fs::read_to_string(root.join(".github/workflows/compact.yml")).unwrap();
    let line = text.lines().nth(compact.line - 1).unwrap();
    assert!(line.contains("docker run -p 5432:5432"), "{line}");
    assert!(!line.trim_start().starts_with("echo"), "{line}");
}

#[test]
fn shell_quotes_substitutions_and_joins_stay_one_command() {
    let script = "echo 'a\nb' \"c\\\nd\" \"e\\f\" \"g`h\ni`\" \"j$(k\n(l))\"\ncurl \\\n  -p 5432:5432 # note\ntrue && echo x; echo y || echo z\necho `m\nn` $(o)\necho \\x \"unterminated";
    let commands = super::shell::commands(script);
    assert!(commands.len() > 3);
    assert!(commands.iter().any(|command| command.dynamic));
    let texts: Vec<_> = commands
        .iter()
        .flat_map(|command| command.words.iter().map(|word| word.text.as_str()))
        .collect();
    assert!(texts.iter().any(|text| text.contains('b')), "{texts:?}");
    assert!(texts.iter().any(|text| text.contains('f')), "{texts:?}");
    assert!(texts.contains(&"5432:5432"), "{texts:?}");
    assert!(texts.contains(&"x"), "{texts:?}");
}

#[test]
fn parser_edges_cover_wrappers_hosts_and_yaml_shapes() {
    use super::mapping::{classify, Verdict};

    for spec in [
        "",
        "1 2",
        "80/http",
        "80/tcp/udp",
        "[]:1:2",
        "a:b:c:d",
        "[::1]:a:b:c",
        "${unclosed:80",
        "$:80",
        "${{PORT}}x:80",
        "${{a${{b}}}}:80",
        "${{ PORT:80",
        "0-0-0:80",
        "1-2-3:80",
        "${a{b}}:80",
        "${}:80",
    ] {
        assert!(matches!(classify(spec), Verdict::Silent), "{spec}");
    }
    for spec in ["[::1]::80", "80", "0-0:80", "00:80", "${{PORT}}:80"] {
        assert!(matches!(classify(spec), Verdict::Allowed), "{spec}");
    }
    assert!(matches!(classify("9:8/sctp"), Verdict::Literal(_)));
    assert!(matches!(classify("[::1]:9:80"), Verdict::Literal(_)));

    assert_eq!(super::flags::short_kind('z'), None);
    assert_eq!(super::flags::long_arg("nope"), None);
    assert!(!super::flags::is_assign("1=x"));
    assert!(!super::flags::is_assign("echo"));
    assert_eq!(super::flags::basename(r"C:\docker"), "docker");

    let hit = |script: &str| super::publish::publish_hits(script);
    assert!(hit("docker run -p 1:2 \"").is_empty());
    assert!(hit("docker build -p 1:2 img").is_empty());
    assert!(hit("docker container build -p 1:2 img").is_empty());
    assert!(hit("docker run -z -p 1:2 img").is_empty());
    assert!(hit("docker run --publish").is_empty());
    assert_eq!(hit("docker run --mystery=1 -p 1:2 img").len(), 1);
    assert_eq!(hit("docker run --rm=true -p 3:4 img").len(), 1);
    assert_eq!(hit("docker run --memory $(echo 1) -p 5:6 img").len(), 1);
    assert_eq!(hit("docker run -p=7:8 img").len(), 1);
    assert_eq!(hit("sudo -E -n -H -u root docker run -p 1:2 img").len(), 1);
    assert!(hit("sudo -u").is_empty());
    assert_eq!(hit("() docker run -p 1:2 img").len(), 1);
    assert_eq!(hit("(FOO=1) docker run -p 2:3 img").len(), 1);
    assert_eq!(hit("time ! { docker run -p 4:5 img; }").len(), 1);
    assert_eq!(
        hit("cat <<\"EOF\"\ninner\nEOF\ndocker run -p 6:7 img").len(),
        1
    );
    assert_eq!(
        hit("cat <<-EOF\n\tinner\nEOF\ndocker run -p 8:9 img").len(),
        1
    );
    assert!(hit("cat <<EOF\ninner").is_empty());
    assert_eq!(hit("echo <<<text\ndocker run -p 1:2 img").len(), 1);
    assert_eq!(hit("echo <(cat) >(tee)\ndocker run -p 1:2 img").len(), 1);
    assert!(hit("echo `unterminated").is_empty());
    assert!(hit("echo $(echo hi").is_empty());
    let quoted = super::shell::commands("echo $(echo 'a)' \"b)\")\necho \"a\nb\" \"trail\\");
    assert!(quoted.len() >= 2);
    assert!(quoted.iter().any(|command| command.dynamic));

    let numbered = "\
on: push
jobs:
  1:
    steps:
      - run: docker run -p 1:2 img
";
    let findings = check(WORKFLOW, numbered);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("1"));

    let boolean = "\
on: push
jobs:
  true:
    steps:
      - run: docker run -p 1:2 img
";
    assert_eq!(check(WORKFLOW, boolean)[0].target.as_deref(), Some("true"));

    let empty_name = "\
on: push
jobs:
  \"\":
    steps:
      - run: docker run -p 1:2 img
";
    assert_eq!(
        check(WORKFLOW, empty_name)[0].target.as_deref(),
        Some("job")
    );

    for source in [
        "on: push\njobs:\n  build: null\n",
        "on: push\njobs:\n  build:\n    runs-on: x\n",
        "on: push\njobs:\n  build:\n    steps:\n      - 1\n",
        "name: a\nruns:\n  using: composite\n",
        "name: a\nruns:\n  using: composite\n  steps:\n    - uses: actions/checkout@v4\n",
    ] {
        let rel = if source.contains("jobs:") {
            WORKFLOW
        } else {
            ACTION
        };
        assert!(check(rel, source).is_empty(), "{source}");
    }

    let folded = "\
on: push
jobs:
  build:
    steps:
      - run: >
          docker run -p
          5432:5432 postgres
";
    let findings = check(WORKFLOW, folded);
    assert_eq!(findings.len(), 1, "{findings:?}");
    let line = folded.lines().nth(findings[0].line - 1).unwrap();
    assert!(line.contains("5432:5432"), "{line}");
}
