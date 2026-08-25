// Lightweight repo-map generator, inspired by Aider's repo-map feature.
// Instead of full tree-sitter parsing, this uses fast regex-based signature
// extraction per language so the LLM gets real structural context about the
// codebase (function/class/struct names) without shipping whole file
// contents on every prompt.
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

const IGNORED_DIRS: [&str; 10] = [
    "node_modules", ".git", ".next", "dist", "build", ".cache", "target", ".venv", "vendor", "__pycache__",
];

const MAX_FILES: usize = 250;
const MAX_BYTES_PER_FILE: u64 = 300_000;
const MAX_OUTPUT_CHARS: usize = 24_000;

struct LangSpec {
    exts: &'static [&'static str],
    patterns: &'static [&'static str],
}

fn lang_specs() -> Vec<LangSpec> {
    vec![
        LangSpec {
            exts: &["rs"],
            patterns: &[
                r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+\w+[^\{;]*",
                r"^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+\w+",
                r"^\s*(?:pub(?:\([^)]*\))?\s+)?enum\s+\w+",
                r"^\s*(?:pub(?:\([^)]*\))?\s+)?trait\s+\w+",
                r"^\s*impl(?:<[^>]*>)?\s+[\w:<>]+",
            ],
        },
        LangSpec {
            exts: &["ts", "tsx", "js", "jsx"],
            patterns: &[
                r"^\s*(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s+\w+\s*\([^)]*\)",
                r"^\s*(?:export\s+)?(?:default\s+)?class\s+\w+",
                r"^\s*(?:export\s+)?(?:const|let)\s+\w+\s*=\s*(?:async\s+)?\([^)]*\)\s*=>",
                r"^\s*(?:export\s+)?interface\s+\w+",
            ],
        },
        LangSpec {
            exts: &["py"],
            patterns: &[
                r"^\s*(?:async\s+)?def\s+\w+\s*\([^)]*\)",
                r"^\s*class\s+\w+",
            ],
        },
        LangSpec {
            exts: &["dart"],
            patterns: &[
                r"^\s*(?:abstract\s+)?class\s+\w+",
                r"^\s*(?:static\s+)?(?:Future<[^>]*>|void|int|double|bool|String|var|final)\s+\w+\s*\([^)]*\)\s*(?:async\s*)?\{?",
            ],
        },
        LangSpec {
            exts: &["go"],
            patterns: &[
                r"^\s*func\s+(?:\([^)]*\)\s*)?\w+\s*\([^)]*\)",
                r"^\s*type\s+\w+\s+struct",
            ],
        },
        LangSpec {
            exts: &["java", "kt"],
            patterns: &[
                r"^\s*(?:public|private|protected)?\s*(?:static\s+)?(?:class|interface)\s+\w+",
                r"^\s*(?:public|private|protected)?\s*(?:static\s+)?[\w<>\[\]]+\s+\w+\s*\([^)]*\)\s*\{",
            ],
        },
    ]
}

fn ext_of(path: &PathBuf) -> String {
    path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

/// Scans the workspace and returns a compact textual "map" listing the
/// signatures (functions, classes, structs, ...) found in each source file.
/// This is real static analysis of files actually on disk — no fabricated
/// symbols — meant to be injected into the LLM system prompt so it has
/// structural awareness of the project without needing to read every file.
#[tauri::command]
pub fn generate_repo_map(path: String) -> Result<String, String> {
    let root = PathBuf::from(&path);
    if !root.exists() || !root.is_dir() {
        return Err(format!("Directory non trovata: {}", path));
    }

    let specs = lang_specs();
    let compiled: Vec<(Vec<&str>, Vec<Regex>)> = specs
        .iter()
        .map(|s| {
            let regexes: Vec<Regex> = s
                .patterns
                .iter()
                .filter_map(|p| Regex::new(p).ok())
                .collect();
            (s.exts.to_vec(), regexes)
        })
        .collect();

    let mut out = String::new();
    let mut files_scanned = 0usize;
    let mut total_symbols = 0usize;

    for entry in WalkDir::new(&root)
        .max_depth(6)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !IGNORED_DIRS.iter().any(|&ign| name == ign)
        })
        .filter_map(|e| e.ok())
    {
        if files_scanned >= MAX_FILES || out.len() >= MAX_OUTPUT_CHARS {
            break;
        }
        if entry.file_type().is_dir() {
            continue;
        }

        let p = entry.path().to_path_buf();
        let ext = ext_of(&p);
        let Some((_, regexes)) = compiled.iter().find(|(exts, _)| exts.contains(&ext.as_str())) else {
            continue;
        };

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.len() == 0 || meta.len() > MAX_BYTES_PER_FILE {
            continue;
        }

        let content = match fs::read_to_string(&p) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut file_symbols = Vec::new();
        for line in content.lines() {
            for re in regexes {
                if let Some(m) = re.find(line) {
                    let mut sig = m.as_str().trim().to_string();
                    if sig.len() > 140 {
                        sig.truncate(140);
                        sig.push('…');
                    }
                    file_symbols.push(sig);
                    break;
                }
            }
        }

        if !file_symbols.is_empty() {
            files_scanned += 1;
            let rel = p.strip_prefix(&root).unwrap_or(&p).to_string_lossy();
            out.push_str(&format!("\n{}\n", rel));
            for sig in &file_symbols {
                total_symbols += 1;
                out.push_str(&format!("  {}\n", sig));
                if out.len() >= MAX_OUTPUT_CHARS {
                    break;
                }
            }
        }
    }

    if out.len() >= MAX_OUTPUT_CHARS {
        out.push_str("\n… (repo map troncata, progetto molto grande) …\n");
    }

    let header = format!(
        "# REPO MAP ({} file, {} simboli reali estratti via regex da '{}')\n",
        files_scanned, total_symbols, path
    );
    Ok(format!("{}{}", header, out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn extracts_real_signatures_from_real_files_on_disk() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("ccc_repomap_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("lib.rs"),
            "pub fn calcola_totale(a: i32, b: i32) -> i32 {\n    a + b\n}\n\npub struct Ordine {\n    id: u32,\n}\n",
        )
        .unwrap();
        fs::write(
            dir.join("app.py"),
            "def somma(a, b):\n    return a + b\n\nclass Cliente:\n    pass\n",
        )
        .unwrap();

        let map = generate_repo_map(dir.to_string_lossy().to_string()).expect("repo map should succeed");

        assert!(map.contains("calcola_totale"), "map missing rust fn: {}", map);
        assert!(map.contains("struct Ordine"), "map missing rust struct: {}", map);
        assert!(map.contains("def somma"), "map missing python def: {}", map);
        assert!(map.contains("class Cliente"), "map missing python class: {}", map);
        assert!(map.starts_with("# REPO MAP ("));

        fs::remove_dir_all(&dir).ok();
    }
}
