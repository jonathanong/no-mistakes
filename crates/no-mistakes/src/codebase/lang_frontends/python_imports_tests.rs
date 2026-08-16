use super::imports::{extract_python_imports, prefix_package, python_module};
use std::path::Path;

#[test]
fn extract_python_imports_covers_unprefixed_and_star_forms() {
    let path = Path::new("/repo/app/users/views.py");
    let imports = extract_python_imports(
        "import app.tasks, app.models\nfrom . import *\nfrom . import (\n    helper,\n)\nfrom ...outside import nope\nfrom app.mod import helper",
        path,
        None,
        None,
    );
    assert!(imports.iter().any(|import| import == "app.tasks"));
    assert!(imports.iter().any(|import| import == "app.mod.helper"));
    let aliased = extract_python_imports("import app.tasks as celery_tasks", path, None, None);
    assert!(aliased
        .iter()
        .any(|import| import == "celery_tasks=app.tasks"));
    assert_eq!(python_module(None, None, path), None);
    assert_eq!(
        prefix_package(None, "users.models".to_string()),
        "users.models"
    );
    let pkg = Path::new("/repo");
    let paren = extract_python_imports(
        "from . import (\n    helper,\n)",
        path,
        Some("app"),
        Some(&pkg.join("app")),
    );
    assert!(paren.iter().any(|import| import.ends_with(".helper")));
    assert_eq!(
        python_module(Some("."), Some(pkg), &pkg.join("app/users.py")).as_deref(),
        Some("app.users")
    );
    assert_eq!(
        prefix_package(Some("."), "app.users".to_string()),
        "app.users"
    );
}
