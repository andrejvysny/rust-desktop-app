//! Check that redistributable assets carry author, source and license metadata.
use std::{path::Path, process::Command};

const EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "ico", "icns", "svg", "pdf", "eps", "icc", "icm", "ttf", "otf", "woff", "woff2",
    "mp4", "wav", "mp3",
];

pub fn is_asset(path: &str) -> bool {
    let file = Path::new(path);
    if file.file_name().is_some_and(|name| name == ".gitkeep" || name == "README.md") {
        return false;
    }
    let extension = file.extension().and_then(|ext| ext.to_str()).map(str::to_ascii_lowercase).unwrap_or_default();
    path.starts_with("assets/") || path.starts_with("docs/images/") || EXTENSIONS.contains(&extension.as_str())
}

fn attributed(path: &str, attribution: &str) -> bool {
    attribution.lines().any(|line| {
        let cells: Vec<&str> = line.trim().trim_matches('|').split('|').map(str::trim).collect();
        cells.first().is_some_and(|cell| *cell == format!("`{path}`"))
            && cells.len() == 4
            && cells.iter().skip(1).all(|cell| !cell.is_empty() && *cell != "-")
    })
}

pub fn missing(files: &[String], attribution: &str) -> Vec<String> {
    files.iter().filter(|path| is_asset(path) && !attributed(path, attribution)).cloned().collect()
}

pub fn run(root: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "-z", "--cached", "--others", "--exclude-standard"])
        .output()
        .map_err(|error| format!("git ls-files: {error}"))?;
    if !output.status.success() {
        return Err(format!("git ls-files failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    let listing = String::from_utf8(output.stdout).map_err(|error| format!("asset path is not UTF-8: {error}"))?;
    let files: Vec<String> = listing.split('\0').filter(|path| !path.is_empty() && root.join(path).is_file()).map(str::to_owned).collect();
    let attribution = std::fs::read_to_string(root.join("ATTRIBUTION.md")).map_err(|error| format!("ATTRIBUTION.md: {error}"))?;
    let missing = missing(&files, &attribution);
    if !missing.is_empty() {
        return Err(format!("assets lack author/source/license rows in ATTRIBUTION.md:\n{}", missing.join("\n")));
    }
    println!("assets: {} files attributed", files.iter().filter(|file| is_asset(file)).count());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attribution_requires_complete_exact_row() {
        let files = vec!["assets/icon.svg".into(), "tests/image.png".into()];
        let complete = "| `assets/icon.svg` | Contributor | Original | MIT |";
        assert_eq!(missing(&files, complete), vec!["tests/image.png"]);
        for incomplete in ["`assets/icon.svg`", "| `assets/icon.svg` | | Original | MIT |", "| `assets/icon.svg` | Contributor | Original | - |"] {
            assert_eq!(missing(&files, incomplete).len(), 2);
        }
    }

    #[test]
    fn code_and_directory_markers_need_no_row() {
        assert!(!is_asset("src/lib.rs"));
        assert!(!is_asset("assets/.gitkeep"));
        assert!(!is_asset("assets/README.md"));
        assert!(is_asset("assets/data.bin"));
        assert!(is_asset("tests/image.PNG"));
    }
}
