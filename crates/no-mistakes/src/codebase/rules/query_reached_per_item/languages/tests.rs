use super::resolve::project;
use super::{collect, known, root_entries, selected_files, sink_list, Ctx, Sink};
use crate::codebase::rules::allow_audit::AllowAudit;
use crate::codebase::rules::path_filter::RulePathFilter;
use crate::config::v2::schema::{
    DotnetProjectConfig, EffectKindConfig, EffectTargetConfig, RuleDef, RuleScope,
};
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use std::path::{Path, PathBuf};

mod regressions;
mod shapes;

fn sinks() -> Vec<Sink> {
    vec![Sink {
        kind: "store".to_string(),
        module: "example.store".to_string(),
        export: "read_row".to_string(),
    }]
}

fn ctx<'a>(sinks: &'a [Sink], policy: Unanalyzable) -> Ctx<'a> {
    Ctx {
        sinks,
        message: "per-item call reaches a configured effect sink; batch the helper, merge the lookup, or add a justified suppression",
        unanalyzable: policy,
        allow: &[],
        usage: None,
    }
}

fn project_ext(source: &str, extension: &str) -> Vec<crate::codebase::rules::RuleFinding> {
    let sinks = sinks();
    project("worker", Some(source), extension, &ctx(&sinks, Unanalyzable::Advise))
}

#[test]
fn brace_languages_prove_one_local_hop_and_report_a_receiver() {
    let source = "fn fetch(item: &str) {\n    example.store.read_row(item);\n}\nfn load(items: &[&str]) {\n    for item in items {\n        fetch(item);\n        item.fetch();\n    }\n}\n";
    for extension in ["go", "rs", "java", "kt", "dart", "swift", "cs"] {
        let findings = project_ext(source, extension);
        assert!(
            findings.iter().any(|finding| {
                finding.causes.is_empty() && finding.message.contains("load → fetch → read_row")
            }),
            "{extension}: {findings:#?}"
        );
        assert!(
            findings.iter().any(|finding| {
                finding.causes.first().is_some_and(|cause| cause.kind == "unresolved-dispatch")
                    && finding.message.contains("item.fetch")
            }),
            "{extension}: {findings:#?}"
        );
    }
}

#[test]
fn python_indent_and_block_languages_keep_the_loop_open() {
    let python = "def fetch(item):\n    return example.store.read_row(item)\n\ndef load(items):\n    for item in items:\n        fetch(item)\n        item.fetch()\n";
    let ruby = "def fetch(item)\n  example.store.read_row(item)\nend\n\ndef load(items)\n  for item in items do\n    if item\n      fetch(item)\n    end\n    item.fetch()\n  end\nend\n";
    let elixir = "defmodule Worker do\n  def fetch(item) do\n    example.store.read_row(item)\n  end\n  def load(items) do\n    for item <- items do\n      fetch(item)\n      item.fetch()\n    end\n  end\nend\n";
    for (extension, source) in [("py", python), ("rb", ruby), ("ex", elixir), ("exs", elixir)] {
        let findings = project_ext(source, extension);
        assert!(
            findings.iter().any(|finding| finding.message.contains("read_row")),
            "{extension}"
        );
        assert!(
            findings.iter().any(|finding| finding.message.contains("item.fetch")),
            "{extension}"
        );
    }
}

#[test]
fn php_qualified_call_and_member_call_stay_distinct() {
    let source = "function fetch($item) {\n    example\\store::read_row($item);\n}\nfunction load($items) {\n    foreach ($items as $item) {\n        fetch($item);\n        $item->fetch();\n    }\n}\n";
    let findings = project_ext(source, "php");
    assert!(findings.iter().any(|finding| finding.message.contains("read_row")));
    assert!(findings.iter().any(|finding| finding.import.as_deref() == Some("$item->fetch")));
}

#[test]
fn direct_sink_clean_call_unresolved_body_and_duplicate_name() {
    let sinks = sinks();
    let direct = project(
        "worker",
        Some("fn load(items: &[&str]) {\n    for item in items {\n        example.store.read_row(item);\n    }\n}\n"),
        "rs",
        &ctx(&sinks, Unanalyzable::Advise),
    );
    assert!(direct[0].message.contains("load → read_row"));
    let foreign = project(
        "worker",
        Some("fn load(items: &[&str]) {\n    for item in items {\n        other.module.lookup(item);\n    }\n}\n"),
        "rs",
        &ctx(&sinks, Unanalyzable::Advise),
    );
    assert!(foreign.iter().any(|finding| {
        finding.causes.first().is_some_and(|cause| cause.kind == "unresolved-dispatch")
            && finding.message.contains("other.module.lookup")
    }));
    let same_module = project(
        "worker",
        Some("fn load(items: &[&str]) {\n    for item in items {\n        example.store.write_row(item);\n    }\n}\n"),
        "rs",
        &ctx(&sinks, Unanalyzable::Advise),
    );
    assert!(same_module.is_empty());
    let unresolved = project_ext(
        "fn fetch(item: &str) {\n    item.other();\n}\nfn load(items: &[&str]) {\n    for item in items {\n        fetch(item);\n    }\n}\n",
        "rs",
    );
    assert!(unresolved.iter().any(|finding| !finding.causes.is_empty()));
    let duplicate = project_ext(
        "fn fetch(item: &str) { example.store.read_row(item); }\nfn fetch(item: &str) { example.store.read_row(item); }\nfn load(items: &[&str]) {\n    for item in items {\n        fetch(item);\n    }\n}\n",
        "rs",
    );
    assert_eq!(duplicate.len(), 1);
    assert_eq!(duplicate[0].causes[0].kind, "unresolved-dispatch");
    assert!(duplicate[0].target.is_none());
    assert!(!duplicate[0].message.contains("read_row"));
}

#[test]
fn each_loop_report_and_ignore_policies_and_allowance() {
    let source = "def fetch(item)\n  example.store.read_row(item)\nend\ndef load(items)\n  items.each do\n    fetch(item)\n  end\nend\n";
    let findings = project_ext(source, "rb");
    assert!(findings.iter().any(|finding| finding.message.contains("each `fetch`")));
    let sinks = sinks();
    let reported = project("worker", Some("fn load(items: &[&str]) {\n    for item in items {\n        item.fetch();\n    }\n}\n"), "rs", &ctx(&sinks, Unanalyzable::Report));
    assert!(reported[0].causes.is_empty());
    let ignored = project("worker", Some("fn load(items: &[&str]) {\n    for item in items {\n        item.fetch();\n    }\n}\n"), "rs", &ctx(&sinks, Unanalyzable::Ignore));
    assert!(ignored.is_empty());
    let unread = project("worker.py", None, "py", &ctx(&sinks, Unanalyzable::Advise));
    assert_eq!(unread[0].causes[0].kind, "unresolved-dispatch");
    let allow = [super::super::config::Allowance {
        file: "worker".to_string(),
        function: Some("load".to_string()),
        callee: Some("item.fetch".to_string()),
        line: Some(3),
    }];
    let mut allowed = ctx(&sinks, Unanalyzable::Advise);
    allowed.allow = &allow;
    let suppressed = project(
        "worker",
        Some("fn load(items: &[&str]) {\n    for item in items {\n        item.fetch();\n        item.load();\n    }\n}\n"),
        "rs",
        &allowed,
    );
    assert!(suppressed.iter().all(|finding| !finding.message.contains("item.fetch")));
    assert!(suppressed.iter().any(|finding| finding.message.contains("item.load")));
}

#[test]
fn language_ids_roots_and_session_requirement() {
    for language in ["python", "go", "rust", "rails", "php", "java", "kotlin", "elixir", "dart", "swift", "dotnet"] {
        assert!(known(language), "{language}");
    }
    assert!(!known("typescript"));
    let mut config = NoMistakesConfig::default();
    config.tests.python.packages = vec!["python".into()];
    config.tests.go.modules = vec!["go".into()];
    config.tests.rust.packages = vec!["rust".into()];
    config.tests.rails.apps = vec!["ruby".into()];
    config.tests.php.apps = vec!["php".into()];
    config.tests.java.packages = vec!["java".into()];
    config.tests.kotlin.packages = vec!["kotlin".into()];
    config.tests.elixir.apps = vec!["elixir".into()];
    config.tests.dart.packages = vec!["dart".into()];
    config.tests.swift.packages = vec!["swift".into()];
    config.tests.dotnet.projects.insert(
        "api".into(),
        DotnetProjectConfig {
            project: "csharp/api.csproj".into(),
            include: Vec::new(),
            exclude: Vec::new(),
            test: false,
        },
    );
    for language in ["python", "go", "rust", "rails", "php", "java", "kotlin", "elixir", "dart", "swift", "dotnet"] {
        assert!(!root_entries(&config, language).is_empty(), "{language}");
    }
    let root = PathBuf::from("/repo");
    let mut rule = RuleDef {
        rule: "query-reached-per-item".into(),
        scope: Some(RuleScope::Repository),
        exclude: vec!["python/**".into()],
        ..RuleDef::default()
    };
    let filter = RulePathFilter::new(&root, &config, &rule).unwrap();
    let visible = vec![root.join("python/worker.py"), root.join("go/worker.go")];
    let excluded = selected_files(&root, &config, &["python".into(), "python".into()], &visible, &filter);
    assert!(excluded.is_empty());
    rule.exclude.clear();
    let filter = RulePathFilter::new(&root, &config, &rule).unwrap();
    let kept = selected_files(&root, &config, &["python".into(), "go".into()], &visible, &filter);
    assert_eq!(kept.len(), 2);
    let dotnet = selected_files(
        &root,
        &config,
        &["dotnet".into()],
        &[root.join("csharp/Worker.cs")],
        &filter,
    );
    assert_eq!(dotnet.len(), 1);
    config.effects.insert("store".into(), EffectKindConfig {
        targets: vec![EffectTargetConfig {
            module: "example.store".into(),
            export: "read_row".into(),
            category: Some("query".into()),
        }],
        ..EffectKindConfig::default()
    });
    let options = super::super::config::Options {
        effects: vec!["store".into()],
        languages: vec!["cobol".into()],
        ..Default::default()
    };
    let error = super::super::config::validate(&options, &config).unwrap_err();
    assert!(error.to_string().contains("unknown options.languages"));
    let options = super::super::config::Options {
        effects: vec!["store".into()],
        languages: vec!["python".into()],
        ..Default::default()
    };
    config.tests.python.packages.clear();
    let error = super::super::config::validate(&options, &config).unwrap_err();
    assert!(error.to_string().contains("configured tests root"));
    config.tests.python.packages = vec!["python".into()];
    let audit = AllowAudit::new(Path::new("."), &config, &rule, None, "allow", 0);
    let options = super::super::config::Options::default();
    assert!(collect(&root, &config, &rule, &options, &audit, None).unwrap().is_empty());
    let options = super::super::config::Options {
        effects: vec!["store".into()],
        languages: vec!["python".into(), "python".into()],
        ..Default::default()
    };
    let error = collect(&root, &config, &rule, &options, &audit, None).unwrap_err();
    assert!(error.to_string().contains("prepared analysis session"));
}

#[test]
fn module_loops_clean_helpers_and_bare_receivers_stay_distinct() {
    assert!(project_ext("example.store.read_row(item);\n", "txt").is_empty());
    let module = project_ext(
        "example.store.read_row(item);\nfor item in items {\n    example.store.read_row(item);\n}\n",
        "rs",
    );
    assert_eq!(module.len(), 1);
    assert!(module[0].causes.is_empty());
    assert!(module[0].message.contains("for `example.store.read_row`"));
    assert!(module[0].message.contains("<module> → read_row"));
    let while_loop = project_ext(
        "fn load(items: &[&str]) {\n    while items {\n        example.store.read_row(item);\n    }\n}\n",
        "rs",
    );
    assert!(while_loop.iter().any(|finding| {
        finding.causes.is_empty() && finding.message.contains("while `example.store.read_row`")
    }));
    let spaced = project_ext(
        "fn load (items: &[&str]) {\n    for item in items {\n        if (ready) {\n            example.store.read_row(item);\n        }\n        (item);\n    }\n}\n",
        "rs",
    );
    assert_eq!(spaced.len(), 1);
    assert!(spaced[0].message.contains("load → read_row"));
    let tail = project_ext(
        "fn load(items: &[&str]) {\n    for item in items {\n        item.();\n    }\n}\n",
        "rs",
    );
    assert!(tail.iter().any(|finding| finding.message.contains("`item.`")));
    let foreign_helper = project_ext(
        "fn fetch(item: &str) {\n    other.module.lookup(item);\n}\nfn load(items: &[&str]) {\n    for item in items {\n        fetch(item);\n    }\n}\n",
        "rs",
    );
    assert!(foreign_helper.iter().any(|finding| {
        finding.causes.first().is_some_and(|cause| cause.kind == "unresolved-dispatch")
            && finding.message.contains("fetch")
    }));
    let doubled = project_ext(
        "fn fetch(item: &str) {\n    example.store.read_row(item);\n    example.store.read_row(item);\n    example.store.write_row(item);\n}\nfn load(items: &[&str]) {\n    for item in items {\n        fetch(item);\n    }\n}\n",
        "rs",
    );
    assert_eq!(doubled.len(), 1);
    assert!(doubled[0].message.contains("load → fetch → read_row"));
    let mixed = project_ext(
        "fn fetch(item: &str) {\n    example.store.read_row(item);\n    self.db.query(item);\n}\nfn load(items: &[&str]) {\n    for item in items {\n        fetch(item);\n    }\n}\n",
        "rs",
    );
    assert_eq!(mixed.len(), 1);
    assert!(mixed[0].causes.is_empty());
    assert!(mixed[0].message.contains("load → fetch → read_row"));
    let ruby = project_ext(
        "def load(items)\n  for item in items do\n    example.store.read_row(item)\n  end\n  def (\n  items.each {\n    item.fetch()\n  }\nend\n",
        "rb",
    );
    assert!(ruby.iter().any(|finding| finding.message.contains("for `example.store.read_row`")));
    assert!(ruby.iter().any(|finding| finding.message.contains("each `item.fetch`")));
}

#[test]
fn else_branches_method_names_and_foreach_keep_their_frames() {
    let ruby = "def load(items)\n  for item in items do\n    if item\n      example.store.read_row(item)\n    elsif item\n      example.store.read_row(item)\n    else\n      example.store.read_row(item)\n    end\n  end\n  example.store.read_row(item)\nend\n";
    let ruby_findings = project_ext(ruby, "rb");
    assert_eq!(ruby_findings.len(), 3);
    assert!(ruby_findings.iter().all(|finding| finding.message.contains("for `example.store.read_row`")));
    let elixir = "defmodule Worker do\n  def load(items) do\n    for item <- items do\n      if item do\n        example.store.read_row(item)\n      else\n        example.store.read_row(item)\n      end\n    end\n    example.store.read_row(item)\n  end\nend\n";
    assert_eq!(project_ext(elixir, "ex").len(), 2);
    let begin = "def load(items)\n  for item in items do\n    begin\n      item.fetch()\n    end\n    value = if item\n      item.load()\n    end\n    example.store.read_row(item) if item\n  end\n  example.store.read_row(item)\nend\n";
    let begun = project_ext(begin, "rb");
    assert_eq!(begun.len(), 3);
    assert!(begun.iter().all(|finding| finding.message.contains("for ")));
    let method = "func (s *Server) load(items []string) {\n    for _, item := range items {\n        example.store.read_row(item)\n    }\n}\n";
    let go = project_ext(method, "go");
    assert!(go.iter().any(|finding| finding.message.contains("load → read_row")));
    assert!(go.iter().all(|finding| !finding.message.contains("func →")));
    let each = "func load(items []string) {\n    items.forEach(func(item string) {\n        example.store.read_row(item)\n    })\n}\n";
    assert!(project_ext(each, "go").iter().any(|finding| finding.message.contains("each `example.store.read_row`")));
    let receiver = project_ext(
        "fn load(items: &[&str]) {\n    for item in items {\n        self.db.query(item);\n        crate::db::query(item);\n    }\n}\n",
        "rs",
    );
    assert_eq!(receiver.len(), 2);
    assert!(receiver.iter().all(|finding| !finding.causes.is_empty()));
}

#[test]
fn unknown_language_missing_effect_and_extensionless_path_are_skipped() {
    let mut config = NoMistakesConfig::default();
    config.tests.python.packages = vec!["python".into()];
    assert!(root_entries(&config, "cobol").is_empty());
    config.tests.dotnet.projects.insert(
        "lib".into(),
        DotnetProjectConfig {
            project: "csharp".into(),
            include: Vec::new(),
            exclude: Vec::new(),
            test: false,
        },
    );
    config.effects.insert(
        "store".into(),
        EffectKindConfig {
            targets: vec![EffectTargetConfig {
                module: "example.store".into(),
                export: "read_row".into(),
                category: Some("query".into()),
            }],
            ..EffectKindConfig::default()
        },
    );
    let options = super::super::config::Options {
        effects: vec!["missing".into(), "store".into()],
        ..Default::default()
    };
    let listed = sink_list(&config, &options);
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].export, "read_row");
    let root = PathBuf::from("/repo");
    let rule = RuleDef {
        rule: "query-reached-per-item".into(),
        scope: Some(RuleScope::Repository),
        ..RuleDef::default()
    };
    let filter = RulePathFilter::new(&root, &config, &rule).unwrap();
    let files = selected_files(
        &root,
        &config,
        &["cobol".into(), "python".into()],
        &[root.join("python/README"), root.join("python/worker.py")],
        &filter,
    );
    assert_eq!(files.len(), 1);
    assert!(files[0].0.ends_with("worker.py"));
    let dotnet = selected_files(
        &root,
        &config,
        &["dotnet".into()],
        &[root.join("csharp/Worker.cs")],
        &filter,
    );
    assert_eq!(dotnet.len(), 1);
}
