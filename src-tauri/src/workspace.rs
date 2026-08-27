use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceContext {
    #[serde(rename = "folderName")]
    pub folder_name: String,
    #[serde(rename = "fullPath")]
    pub full_path: String,
    #[serde(rename = "totalFiles")]
    pub total_files: usize,
    pub frameworks: Vec<String>,
    #[serde(rename = "rulesFileName")]
    pub rules_file_name: String,
    #[serde(rename = "rulesSnippet")]
    pub rules_snippet: String,
    #[serde(rename = "hasRulesFile")]
    pub has_rules_file: bool,
    pub tree: Vec<FileNode>,
}

#[tauri::command]
pub fn scan_workspace(path: String) -> Result<WorkspaceContext, String> {
    let root = PathBuf::from(&path);
    if !root.exists() || !root.is_dir() {
        return Err(format!("Directory non trovata: {}", path));
    }

    let folder_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Workspace".to_string());

    let mut tree = Vec::new();
    let mut frameworks = Vec::new();
    let mut total_files = 0;

    let ignored_names = [
        "node_modules", ".git", ".next", "dist", "build", ".cache", "target", ".venv",
    ];

    for entry in WalkDir::new(&root)
        .max_depth(3)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !ignored_names.iter().any(|&ign| name == ign)
        })
        .filter_map(|e| e.ok())
    {
        if entry.path() == root {
            continue;
        }

        let is_dir = entry.file_type().is_dir();
        let metadata = entry.metadata().ok();
        let size = metadata.map(|m| m.len()).unwrap_or(0);
        let name = entry.file_name().to_string_lossy().to_string();

        if !is_dir {
            total_files += 1;
            match name.as_str() {
                "package.json" => {
                    if !frameworks.contains(&"Node.js".to_string()) {
                        frameworks.push("Node.js".to_string());
                    }
                }
                "Cargo.toml" => {
                    if !frameworks.contains(&"Rust / Cargo".to_string()) {
                        frameworks.push("Rust / Cargo".to_string());
                    }
                }
                "requirements.txt" | "pyproject.toml" => {
                    if !frameworks.contains(&"Python".to_string()) {
                        frameworks.push("Python".to_string());
                    }
                }
                "pubspec.yaml" => {
                    if !frameworks.contains(&"Flutter / Dart".to_string()) {
                        frameworks.push("Flutter / Dart".to_string());
                    }
                }
                _ => {}
            }
        }

        if tree.len() < 150 {
            tree.push(FileNode {
                name,
                path: entry.path().to_string_lossy().to_string(),
                is_directory: is_dir,
                size,
            });
        }
    }

    if frameworks.is_empty() {
        frameworks.push("Generic Codebase".to_string());
    }

    let mut rules_file_name = String::new();
    let mut rules_snippet = String::new();
    let possible_rules = [".cursorrules", "CLAUDE.md", "claude.md", "AGENTS.md", "GEMINI.md", ".windsurfrules"];

    for rf in &possible_rules {
        let p = root.join(rf);
        if p.exists() {
            if let Ok(content) = fs::read_to_string(&p) {
                rules_file_name = rf.to_string();
                rules_snippet = content;
                break;
            }
        }
    }

    let has_rules_file = !rules_file_name.is_empty();

    Ok(WorkspaceContext {
        folder_name,
        full_path: root.to_string_lossy().to_string(),
        total_files,
        frameworks,
        rules_file_name,
        rules_snippet,
        has_rules_file,
        tree,
    })
}

#[tauri::command]
pub fn read_workspace_file(path: String) -> Result<String, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err("File non trovato".to_string());
    }
    fs::read_to_string(p).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_workspace_file(path: String, content: String) -> Result<bool, String> {
    let p = Path::new(&path);
    if let Some(parent) = p.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(p, content).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Applies a unified diff (as typically emitted by an LLM proposing a code
/// change) to a real file on disk. Uses the `diffy` crate to actually parse
/// and apply the patch — this is a genuine implementation of the
/// Aider/Cursor "propose diff -> apply diff" pattern, not just chat text.
fn apply_patch_text(path: &str, diff: &str) -> Result<String, String> {
    let p = Path::new(path);
    let original = if p.exists() {
        fs::read_to_string(p).map_err(|e| format!("Impossibile leggere il file: {}", e))?
    } else {
        String::new()
    };

    let patch = diffy::Patch::from_str(diff)
        .map_err(|e| format!("Diff non valido o malformato: {}", e))?;

    diffy::apply(&original, &patch)
        .map_err(|e| format!("Impossibile applicare la patch (contesto non corrispondente): {}", e))
}

/// Preview-only: computes what the file would look like after the diff is
/// applied, without writing anything to disk. Used by the UI to show a
/// confirmation dialog before the user commits the change.
#[tauri::command]
pub fn preview_diff_apply(path: String, diff: String) -> Result<String, String> {
    apply_patch_text(&path, &diff)
}

/// Actually writes the patched content to disk. Only called after the user
/// has reviewed and confirmed the diff in the UI.
#[tauri::command]
pub fn apply_diff_to_file(path: String, diff: String) -> Result<bool, String> {
    let new_content = apply_patch_text(&path, &diff)?;
    let p = Path::new(&path);
    if let Some(parent) = p.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(p, new_content).map_err(|e| format!("Impossibile scrivere il file: {}", e))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn real_diff_is_applied_to_a_real_file_on_disk() {
        let mut path = env::temp_dir();
        path.push(format!("ccc_diff_test_{}.txt", std::process::id()));
        fs::write(&path, "line one\nline two\nline three\n").unwrap();

        let path_str = path.to_string_lossy().to_string();
        let diff = format!(
            "--- a/{name}\n+++ b/{name}\n@@ -1,3 +1,3 @@\n line one\n-line two\n+line TWO (edited)\n line three\n",
            name = path.file_name().unwrap().to_string_lossy()
        );

        let preview = preview_diff_apply(path_str.clone(), diff.clone()).expect("preview should succeed");
        assert!(preview.contains("line TWO (edited)"));
        assert!(!preview.contains("line two\n"));

        apply_diff_to_file(path_str.clone(), diff).expect("apply should succeed");
        let on_disk = fs::read_to_string(&path).unwrap();
        assert_eq!(on_disk, preview, "file on disk must match previewed content");
        assert!(on_disk.contains("line TWO (edited)"));

        fs::remove_file(&path).ok();
    }

    #[test]
    fn diff_with_mismatched_context_is_rejected_without_touching_the_file() {
        let mut path = env::temp_dir();
        path.push(format!("ccc_diff_test_bad_{}.txt", std::process::id()));
        fs::write(&path, "untouched\n").unwrap();
        let path_str = path.to_string_lossy().to_string();

        // References context lines that do not exist in the real file, so
        // diffy must fail to apply it instead of silently corrupting the file.
        let diff = "--- a/x.txt\n+++ b/x.txt\n@@ -1,3 +1,3 @@\n this line\n-does not exist\n+in the real file\n at all\n";
        let result = apply_diff_to_file(path_str.clone(), diff.to_string());
        assert!(result.is_err(), "expected mismatched-context diff to be rejected");
        assert_eq!(fs::read_to_string(&path).unwrap(), "untouched\n", "file must remain untouched on failure");

        fs::remove_file(&path).ok();
    }

    #[test]
    fn write_then_read_workspace_file_round_trips_real_bytes_on_disk() {
        let mut path = env::temp_dir();
        path.push(format!("ccc_rw_test_{}.txt", std::process::id()));
        let path_str = path.to_string_lossy().to_string();

        write_workspace_file(path_str.clone(), "contenuto reale\ncon più righe\n".to_string())
            .expect("write should succeed");
        let content = read_workspace_file(path_str.clone()).expect("read should succeed");
        assert_eq!(content, "contenuto reale\ncon più righe\n");

        fs::remove_file(&path).ok();
    }

    #[test]
    fn read_workspace_file_errors_on_missing_file() {
        let result = read_workspace_file("/this/path/almost-certainly/does/not/exist/ccc.txt".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn scan_workspace_detects_frameworks_and_lists_real_files() {
        let mut dir = env::temp_dir();
        dir.push(format!("ccc_scan_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        fs::write(dir.join("package.json"), "{}\n").unwrap();
        fs::create_dir_all(dir.join("node_modules")).unwrap();
        fs::write(dir.join("node_modules").join("ignored.js"), "// should not be counted\n").unwrap();

        let ctx = scan_workspace(dir.to_string_lossy().to_string()).expect("scan should succeed");

        assert!(ctx.frameworks.contains(&"Rust / Cargo".to_string()));
        assert!(ctx.frameworks.contains(&"Node.js".to_string()));
        // node_modules is in the ignored_names list, so its file must not be walked.
        assert!(!ctx.tree.iter().any(|n| n.name == "ignored.js"));
        assert_eq!(ctx.total_files, 2, "only Cargo.toml and package.json should be counted");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn scan_workspace_errors_on_missing_directory() {
        let result = scan_workspace("/this/path/almost-certainly/does/not/exist/ccc".to_string());
        assert!(result.is_err());
    }
}

#[tauri::command]
pub fn open_in_editor(editor: String, path: String) -> Result<bool, String> {
    use std::process::Command;
    let target = path.clone();

    #[cfg(target_os = "macos")]
    {
        let res = match editor.as_str() {
            "cursor" => Command::new("open").arg("-a").arg("Cursor").arg(&target).spawn(),
            "code" | "vscode" => Command::new("open").arg("-a").arg("Visual Studio Code").arg(&target).spawn(),
            "windsurf" => Command::new("open").arg("-a").arg("Windsurf").arg(&target).spawn(),
            "finder" => {
                let p = Path::new(&target);
                if p.is_dir() {
                    Command::new("open").arg(&target).spawn()
                } else {
                    Command::new("open").arg("-R").arg(&target).spawn()
                }
            }
            _ => Command::new(&editor).arg(&target).spawn(),
        };
        return res.map(|_| true).map_err(|e| e.to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let res = match editor.as_str() {
            "cursor" => Command::new("cmd").args(&["/C", "cursor", &target]).spawn(),
            "code" | "vscode" => Command::new("cmd").args(&["/C", "code", &target]).spawn(),
            "finder" | "explorer" => Command::new("explorer").arg(&target).spawn(),
            _ => Command::new("cmd").args(&["/C", &editor, &target]).spawn(),
        };
        return res.map(|_| true).map_err(|e| e.to_string());
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let res = match editor.as_str() {
            "cursor" => Command::new("cursor").arg(&target).spawn(),
            "code" | "vscode" => Command::new("code").arg(&target).spawn(),
            "finder" => Command::new("xdg-open").arg(&target).spawn(),
            _ => Command::new(&editor).arg(&target).spawn(),
        };
        return res.map(|_| true).map_err(|e| e.to_string());
    }
}

#[tauri::command]
pub fn pick_folder() -> Result<String, String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("osascript")
            .arg("-e")
            .arg("POSIX path of (choose folder with prompt \"Seleziona la cartella del progetto:\")")
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(path);
            }
        }
        return Err("Nessuna cartella selezionata".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-Command",
                "Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.FolderBrowserDialog; if($f.ShowDialog() -eq 'OK'){ $f.SelectedPath }",
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(path);
            }
        }
        return Err("Nessuna cartella selezionata".to_string());
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let output = Command::new("zenity")
            .arg("--file-selection")
            .arg("--directory")
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(path);
            }
        }
        return Err("Nessuna cartella selezionata".to_string());
    }
}
