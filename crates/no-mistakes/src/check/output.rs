use crate::check_runner::{self, CheckResults};
use no_mistakes::cli::Format;
use no_mistakes::codebase::rules::RuleFinding;
use no_mistakes::codebase::unique_exports::UniqueExportFinding;
use no_mistakes::integration_tests::IntegrationFinding;
use no_mistakes::queue::CheckFinding;
use no_mistakes::react_traits;

pub(super) fn print(results: &CheckResults, format: Format) {
    match format {
        Format::Json => print_check_json(results),
        Format::Yml => print_check_yml(results),
        Format::Md => print_check_md(results),
        Format::Paths => print_check_paths(results),
        Format::Human => print_check_human(results),
    }
}

fn print_check_json(results: &CheckResults) {
    println!(
        "{}",
        serde_json::to_string_pretty(&check_runner::json_value(results))
            .expect("serialization of Rust structs never fails")
    );
}

fn print_check_yml(results: &CheckResults) {
    println!(
        "{}",
        serde_yaml::to_string(&check_runner::json_value(results))
            .expect("serialization of Rust structs never fails")
    );
}

fn print_check_md(results: &CheckResults) {
    println!("# no-mistakes check");
    println!("## react");
    for v in &results.react {
        println!("- `{}` `{}`: {}", v.file, v.component, v.rule);
    }
    println!("## queues");
    for f in &results.queues {
        println!("- `{}`:{} {}", f.file, f.line, f.message);
    }
    println!("## rules");
    for f in &results.rules {
        println!("- `{}`:{} {} {}", f.file, f.line, f.rule, f.message);
    }
    println!("## integration");
    for f in &results.integration {
        println!("- `{}`:{} {}", f.file, f.line, f.message);
    }
    println!("## codebase");
    for f in &results.codebase {
        println!(
            "- `{}`:{} `{}` {}",
            f.file, f.line, f.export_name, f.message
        );
    }
    println!("## advisories");
    for f in &results.advisories {
        println!("- `{}`:{} {} {}", f.file, f.line, f.rule, f.message);
    }
}

fn print_check_paths(results: &CheckResults) {
    for v in &results.react {
        println!("{}", v.file);
    }
    for f in &results.queues {
        println!("{}:{}", f.file, f.line);
    }
    for f in &results.rules {
        println!("{}:{}", f.file, f.line);
    }
    for f in &results.integration {
        println!("{}:{}", f.file, f.line);
    }
    for f in &results.codebase {
        println!("{}:{}:{}", f.file, f.line, f.export_name);
    }
}

fn print_check_human(results: &CheckResults) {
    if !results.react.is_empty() {
        react_traits::print_violations(&results.react);
    }
    print_queue_human(&results.queues);
    print_rules_human(&results.rules);
    print_integration_human(&results.integration);
    print_codebase_human(&results.codebase);
    print_advisories_human(&results.advisories);
}

fn print_queue_human(findings: &[CheckFinding]) {
    for f in findings {
        println!(
            "{}[{}] {}:{} {}",
            f.kind,
            f.job.as_deref().unwrap_or("*"),
            f.file,
            f.line,
            f.message
        );
    }
}

fn print_rules_human(findings: &[RuleFinding]) {
    for f in findings {
        println!("{} {}:{} {}", f.rule, f.file, f.line, f.message);
    }
}

fn print_integration_human(findings: &[IntegrationFinding]) {
    for f in findings {
        println!(
            "integration[{}:{}] {}:{} {}",
            f.framework, f.suite, f.file, f.line, f.message
        );
    }
}

fn print_codebase_human(findings: &[UniqueExportFinding]) {
    for f in findings {
        println!(
            "{}[{}] {}:{} {}",
            f.rule, f.export_name, f.file, f.line, f.message
        );
    }
}

fn print_advisories_human(findings: &[RuleFinding]) {
    for f in findings {
        println!("advisory {} {}:{} {}", f.rule, f.file, f.line, f.message);
    }
}
