use super::{
    enum_block, enum_variants, kebab_case, reachable_cli_pages, read, repo_root, rust_sources,
    subcommand_enums, Path,
};

#[test]
fn cli_commands_have_docs() {
    let root = repo_root();
    let cli_dir = root.join("docs/cli");
    let index = read(&cli_dir.join("README.md"));
    let source_dir = root.join("crates/no-mistakes/src");

    // Inventory every clap subcommand field from source. This includes the
    // top-level `Command` enum and nested enums such as `TestsCommand`; a
    // guard that only knows about main.rs can silently lose `tests plan` and
    // other leaf pages when a command is added or its page is deleted.
    let mut inventories = Vec::new();
    for source_path in rust_sources(&source_dir) {
        let source = read(&source_path);
        for (parent, enum_name) in subcommand_enums(&source) {
            let block = enum_block(&source, &enum_name).unwrap_or_else(|| {
                panic!(
                    "{}: clap subcommand type `{enum_name}` must have an enum body",
                    source_path.display()
                )
            });
            assert!(
                !block.lines().any(|line| line.contains("name =")),
                "{}: a clap command name override needs an explicit docs-coverage mapping",
                source_path.display()
            );
            let variants = enum_variants(block);
            assert!(
                !variants.is_empty(),
                "{}: {enum_name} command inventory must not be empty",
                source_path.display()
            );
            inventories.push((parent, variants));
        }
    }
    assert!(
        !inventories.is_empty(),
        "source inventory must find at least one clap subcommand enum"
    );

    for (parent, variants) in inventories {
        let Some(prefix) = parent.strip_suffix("Args").map(kebab_case) else {
            // `Cli` is the one top-level parser struct; its command pages are
            // rooted directly at docs/cli and have no group prefix.
            assert_eq!(parent, "Cli", "unexpected clap parser struct `{parent}`");
            for variant in variants {
                assert_cli_page(&cli_dir, &index, &variant, None, 1);
            }
            continue;
        };

        let group_file = format!("{prefix}.md");
        let group_path = cli_dir.join(&group_file);
        assert!(
            group_path.exists(),
            "missing CLI group doc {}",
            group_path.display()
        );
        assert!(
            index.contains(&format!("({group_file})")),
            "docs/cli/README.md must index {group_file}"
        );

        let variant_count = variants.len();
        for variant in variants {
            assert_cli_page(
                &cli_dir,
                &read(&group_path),
                &variant,
                Some(&prefix),
                variant_count,
            );
        }
    }

    // Every leaf page must be reachable from the CLI index or its command
    // group page. Follow only links rooted under docs/cli so an orphan page
    // cannot make itself appear reachable by containing its own filename.
    let linked_pages = reachable_cli_pages(&cli_dir);
    for entry in std::fs::read_dir(root.join("docs/cli")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md")
            || path.file_name().and_then(|name| name.to_str()) == Some("README.md")
        {
            continue;
        }
        let file = path.file_name().unwrap().to_string_lossy();
        assert!(
            linked_pages.contains(file.as_ref()),
            "CLI page {file} is not linked by a CLI index or command group"
        );
    }
}

fn assert_cli_page(
    cli_dir: &Path,
    parent_body: &str,
    variant: &str,
    prefix: Option<&str>,
    variant_count: usize,
) {
    let variant = kebab_case(variant);
    let (file, indexed_by_parent) = match prefix {
        Some(prefix) => {
            let leaf_file = format!("{prefix}-{variant}.md");
            let leaf_path = cli_dir.join(&leaf_file);
            if leaf_path.exists() {
                (leaf_file, true)
            } else {
                // A one-command group may document its only leaf directly on
                // the group page (currently `lockfile diff`). If a second
                // variant is added, the caller's per-variant lookup will no
                // longer permit this fallback without a dedicated leaf page.
                let group_file = format!("{prefix}.md");
                assert_cli_group_has_one_leaf(parent_body, prefix, &group_file, variant_count);
                (group_file, false)
            }
        }
        None => (format!("{variant}.md"), true),
    };
    let path = cli_dir.join(&file);
    assert!(path.exists(), "missing CLI doc {}", path.display());
    if indexed_by_parent {
        assert!(
            parent_body.contains(&format!("({file})")),
            "CLI group doc must index {file}"
        );
    }
}

fn assert_cli_group_has_one_leaf(
    parent_body: &str,
    prefix: &str,
    group_file: &str,
    variant_count: usize,
) {
    assert_eq!(
        variant_count, 1,
        "{group_file} has multiple subcommands; every leaf needs a dedicated CLI page"
    );
    let linked_children = parent_body
        .lines()
        .filter_map(|line| {
            let target = line.split_once("](")?.1.split_once(')')?.0;
            Some(target.to_string())
        })
        .filter(|target| target.ends_with(".md") && target.starts_with(&format!("{prefix}-")))
        .count();
    assert_eq!(
        linked_children, 0,
        "{group_file} has linked leaf pages; missing {prefix}-<command>.md must be fixed explicitly"
    );
}
