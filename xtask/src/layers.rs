//! Explicit dependency boundaries, including build, dev and target dependencies.
use serde_json::Value;

const TABLE: &[(&str, &[&str])] = &[
    ("app-core", &[]),
    ("app-model", &["app-core"]),
    ("app-engine", &["app-core", "app-model"]),
    ("app-platform", &["app-core", "app-engine"]),
    ("app-mcp", &["app-core", "app-engine", "app-platform"]),
    ("app-ui-egui", &["app-core", "app-engine", "app-platform"]),
    ("app-testkit", &["app-core", "app-model", "app-engine", "app-platform"]),
    ("app", &["app-core", "app-engine", "app-platform", "app-ui-egui"]),
    ("app-cli", &["app-core", "app-engine", "app-platform", "app-mcp"]),
    ("app-web", &["app-core", "app-engine", "app-platform", "app-ui-egui"]),
    ("xtask", &[]),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Normal,
    Dev,
    Build,
}

#[derive(Clone, Debug)]
pub struct Dependency {
    pub name: String,
    pub kind: Kind,
    pub local: bool,
}

#[derive(Clone, Debug)]
pub struct Package {
    pub name: String,
    pub dependencies: Vec<Dependency>,
}

fn toolkit(name: &str) -> bool {
    [
        "egui",
        "eframe",
        "winit",
        "rfd",
        "bevy",
        "arboard",
        "objc",
        "objc2",
        "cocoa",
        "windows",
        "gtk",
        "gtk4",
        "gdk4",
        "iced",
        "slint",
        "fltk",
        "wry",
        "web-sys",
        "js-sys",
        "wasm-bindgen",
    ]
    .iter()
    .any(|prefix| name == *prefix || name.starts_with(&format!("{prefix}-")) || name.starts_with(&format!("{prefix}_")))
}

fn ui_allowed(name: &str) -> bool {
    matches!(name, "app-ui-egui" | "app" | "app-web")
}

fn check_dependency(package: &Package, dep: &Dependency, allowed: &[&str]) -> Option<String> {
    if dep.name == package.name && dep.kind == Kind::Dev {
        return None;
    }
    if dep.local {
        if dep.name == "app-testkit" && dep.kind == Kind::Dev && package.name != "xtask" {
            return None;
        }
        if !allowed.contains(&dep.name.as_str()) {
            return Some(format!("{} -> {} ({:?}): workspace dependency not allowed", package.name, dep.name, dep.kind));
        }
    } else if !ui_allowed(&package.name) && toolkit(&dep.name) {
        return Some(format!("{} -> {} ({:?}): UI/platform toolkit forbidden", package.name, dep.name, dep.kind));
    }
    None
}

pub fn check(packages: &[Package]) -> Vec<String> {
    let mut violations = Vec::new();
    for package in packages {
        let Some((_, allowed)) = TABLE.iter().find(|(name, _)| *name == package.name) else {
            violations.push(format!("{}: unknown workspace package; register it in xtask/src/layers.rs", package.name));
            continue;
        };
        violations.extend(package.dependencies.iter().filter_map(|dep| check_dependency(package, dep, allowed)));
    }
    violations.sort();
    violations.dedup();
    violations
}

pub fn from_metadata(metadata: &Value) -> Result<Vec<Package>, String> {
    let packages = metadata.get("packages").and_then(Value::as_array).ok_or("metadata: missing packages")?;
    let names: Vec<&str> = packages.iter().filter_map(|p| p.get("name").and_then(Value::as_str)).collect();
    let mut result = Vec::new();
    for package in packages {
        let name = package.get("name").and_then(Value::as_str).ok_or("metadata: missing package name")?.to_owned();
        let dependencies = package.get("dependencies").and_then(Value::as_array).ok_or("metadata: missing dependencies")?;
        let mut deps = Vec::new();
        for dep in dependencies {
            let name = dep.get("name").and_then(Value::as_str).ok_or("metadata: missing dependency name")?.to_owned();
            let kind = match dep.get("kind").and_then(Value::as_str) {
                Some("dev") => Kind::Dev,
                Some("build") => Kind::Build,
                _ => Kind::Normal,
            };
            let local = names.contains(&name.as_str()) || dep.get("path").is_some_and(Value::is_string);
            deps.push(Dependency { name, kind, local });
        }
        result.push(Package { name, dependencies: deps });
    }
    Ok(result)
}

pub fn run(metadata: &Value) -> Result<(), String> {
    let packages = from_metadata(metadata)?;
    let violations = check(&packages);
    if !violations.is_empty() {
        return Err(format!("dependency layering failed:\n{}", violations.join("\n")));
    }
    println!("layers: {} registered packages, no violations", packages.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(from: &str, to: &str, kind: Kind, local: bool) -> Vec<Package> {
        vec![Package { name: from.into(), dependencies: vec![Dependency { name: to.into(), kind, local }] }]
    }

    #[test]
    fn explicit_edges_pass() {
        for (name, deps) in TABLE {
            for dep in *deps {
                assert!(check(&graph(name, dep, Kind::Normal, true)).is_empty());
            }
        }
    }

    #[test]
    fn upward_sideways_and_app_edges_fail() {
        for (from, to) in [("app-core", "app-model"), ("app-mcp", "app-ui-egui"), ("app-cli", "app-ui-egui"), ("xtask", "app-engine")] {
            assert!(!check(&graph(from, to, Kind::Normal, true)).is_empty());
        }
    }

    #[test]
    fn all_dependency_kinds_enforce_ui_boundary() {
        for kind in [Kind::Normal, Kind::Dev, Kind::Build] {
            for (from, to) in [
                ("app-model", "egui"),
                ("app-core", "rfd"),
                ("app-engine", "eframe"),
                ("app-platform", "arboard"),
                ("app-cli", "winit"),
                ("app-engine", "objc2-app-kit"),
                ("app-model", "gtk4"),
            ] {
                assert!(!check(&graph(from, to, kind.clone(), false)).is_empty());
            }
            assert!(!check(&graph("app-core", "app-model", kind, true)).is_empty());
        }
    }

    #[test]
    fn testkit_is_dev_only_and_cannot_depend_on_frontends() {
        assert!(check(&graph("app-engine", "app-testkit", Kind::Dev, true)).is_empty());
        for kind in [Kind::Normal, Kind::Build] {
            assert!(!check(&graph("app-engine", "app-testkit", kind, true)).is_empty());
        }
        assert!(!check(&graph("app-testkit", "app-ui-egui", Kind::Normal, true)).is_empty());
    }

    #[test]
    fn unknown_package_fails() {
        assert!(!check(&[Package { name: "surprise".into(), dependencies: vec![] }]).is_empty());
    }

    #[test]
    fn target_specific_dependencies_are_checked() {
        let meta = serde_json::json!({"packages": [{"name": "app-engine", "dependencies": [
            {"name": "web-sys", "kind": "build", "target": "cfg(target_arch = \"wasm32\")"}
        ]}]});
        let packages = from_metadata(&meta).unwrap();
        assert_eq!(check(&packages).len(), 1);
    }
}
