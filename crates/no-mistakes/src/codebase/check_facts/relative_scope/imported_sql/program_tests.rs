use super::{frame::helper_program, hole_text, Frame, Locator, Shared};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::extract::{
    ExportedBinding, ImportedBinding, ImportedBindingKind,
};
use crate::codebase::postgres::{
    Callee, Callees, EmbeddedSqlOptions, ImportedSite, Outcome, RecoveredSql, Ret, MAX_CALL_DEPTH,
};
use crate::codebase::ts_source::FileIdMap;
use std::path::PathBuf;

fn sql(text: &str) -> RecoveredSql {
    RecoveredSql {
        sql: text.to_string(),
        line: 1,
        positions: Vec::new(),
        origins: vec![0; text.len()],
    }
}

#[test]
fn hole_text_accepts_a_builder_suffix_and_a_sql_return() {
    let suffix = sql(" AND account_id = $1");
    let appended = Outcome {
        ret: Ret::Param(0),
        suffixes: vec![(0, suffix.clone())],
    };
    assert_eq!(hole_text(appended, Some(0), false).unwrap().sql, suffix.sql);
    let mismatched = Outcome {
        ret: Ret::Void,
        suffixes: Vec::new(),
    };
    assert!(hole_text(mismatched, Some(0), true).is_none());
    let built = Outcome {
        ret: Ret::Sql(sql("SELECT 1")),
        suffixes: Vec::new(),
    };
    assert_eq!(hole_text(built, None, true).unwrap().sql, "SELECT 1");
    let other = Outcome {
        ret: Ret::Void,
        suffixes: Vec::new(),
    };
    assert!(hole_text(other, None, true).is_none());
}

#[test]
fn frame_call_fails_on_depth_cycles_and_cached_failure() {
    let path = PathBuf::from("/mod.mts");
    let mut files = FileIdMap::default();
    files.insert(path.clone(), CheckFileFacts::default());
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let resolve = |_specifier: &str, _from: &std::path::Path| -> Option<PathBuf> { None };

    {
        let locator = Locator::new(&files, resolve);
        let mut shared = Shared::new(&files, locator);
        let mut deep = Frame {
            shared: &mut shared,
            profile: 0,
            options: &options,
            root: path.clone(),
            file: path.clone(),
            depth: MAX_CALL_DEPTH,
            stack: Vec::new(),
        };
        assert!(matches!(deep.call("h", &[]), Callee::Failed));
    }
    {
        let locator = Locator::new(&files, resolve);
        let mut shared = Shared::new(&files, locator);
        let mut cyclic = Frame {
            shared: &mut shared,
            profile: 0,
            options: &options,
            root: path.clone(),
            file: path.clone(),
            depth: 0,
            stack: vec![(path.clone(), "h".to_string())],
        };
        assert!(matches!(cyclic.call("h", &[]), Callee::Failed));
    }
    {
        let locator = Locator::new(&files, resolve);
        let mut shared = Shared::new(&files, locator);
        shared.cache.insert(
            (0, path.clone(), path.clone(), "h".to_string(), Vec::new()),
            None,
        );
        let mut cached = Frame {
            shared: &mut shared,
            profile: 0,
            options: &options,
            root: path.clone(),
            file: path.clone(),
            depth: 0,
            stack: Vec::new(),
        };
        assert!(matches!(cached.call("h", &[]), Callee::Failed));
    }
}

#[test]
fn missing_modules_and_void_programs_fail_closed() {
    let path = PathBuf::from("/mod.mts");
    let mut files = FileIdMap::default();
    files.insert(path.clone(), CheckFileFacts::default());
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let locator = Locator::new(&files, |_specifier: &str, _from: &std::path::Path| {
        Option::<PathBuf>::None
    });
    let mut shared = Shared::new(&files, locator);
    assert!(shared
        .find(std::path::Path::new("/missing.mts"), "h", None)
        .is_none());
    assert!(helper_program(&shared, &options, std::path::Path::new("/missing.mts"), "h").is_none());
    assert!(helper_program(&shared, &options, &path, "h").is_none());
    shared.cache.insert(
        (0, path.clone(), path.clone(), "h".to_string(), Vec::new()),
        Some(vec![Outcome {
            ret: Ret::Void,
            suffixes: Vec::new(),
        }]),
    );
    let site = ImportedSite {
        local: "h".to_string(),
        member: None,
        args: Vec::new(),
        value_used: true,
        line: 1,
        constant: None,
    };
    assert!(shared.fill(0, &options, &path, &site).is_none());
    let repeated = sql("SELECT 1");
    shared.cache.insert(
        (0, path.clone(), path.clone(), "dup".to_string(), Vec::new()),
        Some(vec![
            Outcome {
                ret: Ret::Str("SELECT 1".to_string(), Some(repeated.clone())),
                suffixes: Vec::new(),
            },
            Outcome {
                ret: Ret::Str("SELECT 1".to_string(), Some(repeated)),
                suffixes: Vec::new(),
            },
        ]),
    );
    let duplicate = ImportedSite {
        local: "dup".to_string(),
        ..site
    };
    let filled = shared.fill(0, &options, &path, &duplicate).unwrap();
    assert_eq!(filled.len(), 1);
    // A proven throw has no replacement SQL. It must not become an empty
    // imported-hole variant in the public extraction result.
    shared.cache.insert(
        (
            0,
            path.clone(),
            path.clone(),
            "abrupt".to_string(),
            Vec::new(),
        ),
        Some(Vec::new()),
    );
    let abrupt = ImportedSite {
        local: "abrupt".to_string(),
        ..duplicate
    };
    assert!(shared.fill(0, &options, &path, &abrupt).is_none());
}

#[path = "program_tests/projection.rs"]
mod projection;
