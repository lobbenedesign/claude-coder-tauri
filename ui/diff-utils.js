// Pure, provider-agnostic diff-parsing helpers used by the "propose diff ->
// review -> apply" UI flow (renderDiffProposals / buildDiffCard in app.js).
// Kept in a separate file, deliberately with NO ES module `export` syntax,
// because app.js is loaded as a classic (non-module) <script> in index.html
// and this file is loaded the same way — turning it into a module would
// break that load. Instead this uses a tiny UMD-style shim so the exact
// same functions are:
//   - available as `window.DiffUtils.*` in the real desktop app (browser
//     classic-script context), and
//   - importable via `require()`/`import` in Node for real unit tests
//     (see src-tauri-tests are Rust; this one is exercised by
//     ui/diff-utils.test.js via vitest).
//
// No behavior here is hypothetical: this is the exact logic app.js uses to
// figure out which file on disk a ```diff block from the LLM targets, and
// to resolve that (often relative) path against the attached workspace
// before calling the real `preview_diff_apply` / `apply_diff_to_file`
// Tauri commands.

/// Extracts the target file path from a unified diff's header lines.
/// Prefers the "+++ b/<path>" (new-file) line; falls back to the
/// "--- a/<path>" (old-file) line for pure deletions. Returns null when
/// neither header names a real path (e.g. both are /dev/null, or the diff
/// text has no recognizable header at all).
function extractDiffTargetPath(diffText) {
  const lines = diffText.split("\n");
  for (const line of lines) {
    if (line.startsWith("+++ ")) {
      let p = line.slice(4).trim();
      p = p.replace(/^b\//, "").split("\t")[0].trim();
      if (p && p !== "/dev/null") return p;
    }
  }
  for (const line of lines) {
    if (line.startsWith("--- ")) {
      let p = line.slice(4).trim();
      p = p.replace(/^a\//, "").split("\t")[0].trim();
      if (p && p !== "/dev/null") return p;
    }
  }
  return null;
}

/// Resolves a (possibly relative) diff target path against the attached
/// workspace root. Absolute POSIX paths ("/...") and Windows drive paths
/// ("C:\...") are returned unchanged; anything else is joined onto
/// `workspaceRoot`.
function resolveDiffPath(relOrAbs, workspaceRoot) {
  if (!relOrAbs) return relOrAbs;
  if (relOrAbs.startsWith("/") || /^[A-Za-z]:\\/.test(relOrAbs)) return relOrAbs;
  const sep = workspaceRoot && workspaceRoot.endsWith("/") ? "" : "/";
  return `${workspaceRoot || ""}${sep}${relOrAbs}`;
}

const DiffUtils = { extractDiffTargetPath, resolveDiffPath };

if (typeof window !== "undefined") {
  window.DiffUtils = DiffUtils;
}
if (typeof module !== "undefined" && module.exports) {
  module.exports = DiffUtils;
}
