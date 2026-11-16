use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) fn collect(
    root: &Path,
    sources: &[(PathBuf, bool)],
) -> Result<BTreeMap<String, usize>, String> {
    let mut walker = Walker {
        root: root.canonicalize().map_err(|error| error.to_string())?,
        registrations: BTreeMap::new(),
        stack: BTreeSet::new(),
        module: regex::Regex::new(r#"(?m)^\s*#\[path = "([^"]+\.rs)"\]"#).unwrap(),
        include: regex::Regex::new(r#"(?m)^\s*include!\(\s*"([^"]+)"\s*\)"#).unwrap(),
    };
    for (source, modules) in sources {
        walker.visit(source, *modules)?;
    }
    Ok(walker.registrations)
}

struct Walker {
    root: PathBuf,
    registrations: BTreeMap<String, usize>,
    stack: BTreeSet<PathBuf>,
    module: regex::Regex,
    include: regex::Regex,
}

impl Walker {
    fn visit(&mut self, source: &Path, modules: bool) -> Result<(), String> {
        let source = source
            .canonicalize()
            .map_err(|error| format!("{}: {error}", source.display()))?;
        let Ok(relative) = source.strip_prefix(&self.root) else {
            return Ok(());
        };
        if relative.starts_with("support") || relative.starts_with("common") {
            return Ok(());
        }
        if !self.stack.insert(source.clone()) {
            return Err(format!(
                "recursive integration source: {}",
                source.display()
            ));
        }
        let key = relative.to_string_lossy().replace('\\', "/");
        if source
            .extension()
            .is_some_and(|extension| extension == "rs")
            && (relative.components().count() == 1
                || (relative.components().count() == 2 && source.file_name().unwrap() == "main.rs"))
        {
            *self.registrations.entry(key).or_default() += 1;
        }
        let text = std::fs::read_to_string(&source)
            .map_err(|error| format!("{}: {error}", source.display()))?;
        let mut children: Vec<_> = self
            .include
            .captures_iter(&text)
            .map(|capture| source.parent().unwrap().join(&capture[1]))
            .collect();
        if modules {
            children.extend(
                self.module
                    .captures_iter(&text)
                    .map(|capture| source.parent().unwrap().join(&capture[1])),
            );
        }
        for child in children {
            self.visit(&child, modules)?;
        }
        self.stack.remove(&source);
        Ok(())
    }
}
