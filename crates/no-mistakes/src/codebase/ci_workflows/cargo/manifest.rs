use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;

pub fn parse_cargo_workspace_members(cargo_toml: &str) -> Result<Vec<String>> {
    #[derive(Deserialize, Default)]
    struct CargoToml {
        workspace: Option<Workspace>,
    }
    #[derive(Deserialize, Default)]
    struct Workspace {
        members: Option<Vec<String>>,
    }

    let ct: CargoToml = toml::from_str(cargo_toml)?;
    Ok(ct.workspace.and_then(|ws| ws.members).unwrap_or_default())
}

pub fn parse_cargo_workspace_excludes(cargo_toml: &str) -> Result<Vec<String>> {
    #[derive(Deserialize, Default)]
    struct CargoToml {
        workspace: Option<Workspace>,
    }
    #[derive(Deserialize, Default)]
    struct Workspace {
        exclude: Option<Vec<String>>,
    }

    let ct: CargoToml = toml::from_str(cargo_toml)?;
    Ok(ct.workspace.and_then(|ws| ws.exclude).unwrap_or_default())
}

pub fn parse_cargo_package_name(cargo_toml: &str) -> Result<Option<String>> {
    #[derive(Deserialize, Default)]
    struct CargoToml {
        package: Option<Package>,
    }
    #[derive(Deserialize)]
    struct Package {
        name: String,
    }

    let ct: CargoToml = toml::from_str(cargo_toml)?;
    Ok(ct.package.map(|package| package.name))
}

pub fn parse_cargo_bins(cargo_toml: &str) -> Result<HashMap<String, String>> {
    #[derive(Deserialize, Default)]
    struct CargoToml {
        package: Option<Package>,
        bin: Option<Vec<BinEntry>>,
    }
    #[derive(Deserialize)]
    struct Package {
        name: String,
        autobins: Option<bool>,
    }
    #[derive(Deserialize)]
    struct BinEntry {
        name: String,
        path: Option<String>,
    }

    let ct: CargoToml = toml::from_str(cargo_toml)?;
    let mut map = HashMap::new();
    let package = ct.package;
    let package_name = package.as_ref().map(|pkg| pkg.name.as_str());
    if let Some(package) = &package {
        if package.autobins.unwrap_or(true) {
            map.insert(package.name.clone(), "src/main.rs".to_string());
        }
    }

    for entry in ct.bin.unwrap_or_default() {
        let path = entry.path.unwrap_or_else(|| {
            if package_name == Some(entry.name.as_str()) {
                "src/main.rs".to_string()
            } else {
                format!("src/bin/{}.rs", entry.name)
            }
        });
        map.insert(entry.name, path);
    }
    Ok(map)
}
