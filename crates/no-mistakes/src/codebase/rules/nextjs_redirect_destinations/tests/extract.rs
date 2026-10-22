use super::super::extract::{extract_named_destinations_with, ExtractedDestinations};
use super::super::scan::{check_named_section, NamedCheck};
use crate::config::v2::EffectiveRuleAnalysis;
use std::collections::BTreeSet;
use std::path::Path;

fn extract_named_destinations(path: &Path, source: &str, name: &str) -> ExtractedDestinations {
    extract_named_destinations_with(path, source, name, false)
}

#[test]
fn substring_redirects_text_is_not_extractor_drift() {
    let extracted = extract_named_destinations(
        Path::new("next.config.ts"),
        "export default { preredirects() { return []; } };\n",
        "redirects",
    );
    assert!(!extracted.body_found);
    let findings = check_named_section(NamedCheck {
        rel: "next.config.ts",
        source: "export default { preredirects() { return []; } };\n",
        config_path: Path::new("next.config.ts"),
        name: "redirects",
        kind: "redirect",
        route_set: &BTreeSet::new(),
        app_root_label: "app",
        analysis: EffectiveRuleAnalysis::default(),
    })
    .findings;
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn extract_reads_object_methods_arrows_and_class_members() {
    let object = extract_named_destinations(
        Path::new("next.config.ts"),
        r#"
const nextConfig = {
  other: { keep: true },
  async redirects() {
    return [{ source: "/old", destination: "/about", permanent: true }];
  },
};
export default nextConfig;
"#,
        "redirects",
    );
    assert!(object.body_found);
    assert_eq!(
        object
            .destinations
            .iter()
            .map(|destination| destination.value.as_str())
            .collect::<Vec<_>>(),
        ["/about"]
    );

    let arrow = extract_named_destinations(
        Path::new("next.config.ts"),
        r#"
export default {
  redirects: () => [{ destination: "/login" }],
};
"#,
        "redirects",
    );
    assert!(arrow.body_found);
    assert_eq!(arrow.destinations[0].value, "/login");

    let function_expr = extract_named_destinations(
        Path::new("next.config.ts"),
        r#"
export default {
  redirects: function () {
    return [{ destination: "/docs" }];
  },
};
"#,
        "redirects",
    );
    assert!(function_expr.body_found);
    assert_eq!(function_expr.destinations[0].value, "/docs");

    let class_source = r#"
class Config {
  other() {}
  extra = 1;
  async redirects() {
    return [{ destination: "/about" }];
  }
  rewrites = async () => [{ destination: "/login" }];
}
"#;
    let class_redirects =
        extract_named_destinations(Path::new("next.config.ts"), class_source, "redirects");
    let class_rewrites =
        extract_named_destinations(Path::new("next.config.ts"), class_source, "rewrites");
    assert!(class_redirects.body_found);
    assert!(class_rewrites.body_found);
    assert_eq!(class_redirects.destinations[0].value, "/about");
    assert_eq!(class_rewrites.destinations[0].value, "/login");
}

#[test]
fn extract_walks_nested_objects_and_ignores_spreads() {
    let nested = extract_named_destinations(
        Path::new("next.config.ts"),
        r#"
export default {
  nested: {
    async redirects() {
      return [
        { ...extra },
        { destination: `/dynamic` },
        { destination: "/about" as const },
      ];
    },
  },
};
"#,
        "redirects",
    );
    assert!(nested.body_found);
    assert!(nested.saw_destination_property);
    assert_eq!(
        nested
            .destinations
            .iter()
            .map(|destination| destination.value.as_str())
            .collect::<Vec<_>>(),
        ["/about", "/dynamic"]
    );
}

#[test]
fn extract_keeps_the_first_matching_body() {
    let extracted = extract_named_destinations(
        Path::new("next.config.ts"),
        r#"
export default {
  async redirects() {
    return [{ destination: "/first" }];
  },
  extra: {
    async redirects() {
      return [{ destination: "/second" }];
    },
  },
};
"#,
        "redirects",
    );
    assert_eq!(
        extracted
            .destinations
            .iter()
            .map(|destination| destination.value.as_str())
            .collect::<Vec<_>>(),
        ["/first"]
    );
}

#[test]
fn extract_returns_default_on_parse_failure() {
    let extracted =
        extract_named_destinations(Path::new("next.config.ts"), "not { valid", "redirects");
    assert_eq!(extracted, ExtractedDestinations::default());
}
