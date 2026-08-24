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
