use super::kind::Frontend;
use super::native::configured;
use crate::config::v2::schema::DotnetProjectConfig;
use crate::config::v2::NoMistakesConfig;

#[test]
fn dotnet_is_configured_from_solutions_when_projects_are_empty() {
    let empty = NoMistakesConfig::default();
    let mut solutions = empty.clone();
    solutions.tests.dotnet.solutions.push("App.sln".to_string());
    let mut projects = empty.clone();
    projects.tests.dotnet.projects.insert(
        "app".to_string(),
        DotnetProjectConfig {
            project: "App.csproj".to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            test: false,
        },
    );
    assert_eq!(
        (
            configured(&empty, &Frontend::Dotnet),
            configured(&solutions, &Frontend::Dotnet),
            configured(&projects, &Frontend::Dotnet),
        ),
        (false, true, true)
    );
}
