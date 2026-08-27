// Real unit tests for the pure diff-parsing helpers used by the frontend's
// "propose diff -> review -> apply" flow (ui/diff-utils.js, consumed by
// ui/app.js's renderDiffProposals/buildDiffCard). These do not touch the
// Tauri bridge, the DOM, or any LLM provider — they exercise exactly the
// same path-extraction logic the real app runs on a live LLM response.
import { describe, it, expect } from "vitest";
import DiffUtils from "./diff-utils.js";

const { extractDiffTargetPath, resolveDiffPath } = DiffUtils;

describe("extractDiffTargetPath", () => {
  it("extracts the path from the +++ (new file) header", () => {
    const diff = "--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,2 +1,2 @@\n-old\n+new\n";
    expect(extractDiffTargetPath(diff)).toBe("src/main.rs");
  });

  it("strips the b/ prefix and trailing tab metadata", () => {
    const diff = "--- a/lib.rs\n+++ b/lib.rs\t2024-01-01 00:00:00\n@@ -1 +1 @@\n-x\n+y\n";
    expect(extractDiffTargetPath(diff)).toBe("lib.rs");
  });

  it("falls back to the --- (old file) header for pure deletions", () => {
    const diff = "--- a/old_file.py\n+++ /dev/null\n@@ -1 +0,0 @@\n-removed line\n";
    expect(extractDiffTargetPath(diff)).toBe("old_file.py");
  });

  it("returns null when both headers point at /dev/null", () => {
    const diff = "--- /dev/null\n+++ /dev/null\n";
    expect(extractDiffTargetPath(diff)).toBeNull();
  });

  it("returns null when there is no recognizable diff header at all", () => {
    expect(extractDiffTargetPath("just some random text\nwith no diff markers\n")).toBeNull();
  });

  it("returns null for an empty string", () => {
    expect(extractDiffTargetPath("")).toBeNull();
  });
});

describe("resolveDiffPath", () => {
  it("joins a relative path onto the workspace root", () => {
    expect(resolveDiffPath("src/main.rs", "/Users/dev/project")).toBe("/Users/dev/project/src/main.rs");
  });

  it("does not double a trailing slash on the workspace root", () => {
    expect(resolveDiffPath("src/main.rs", "/Users/dev/project/")).toBe("/Users/dev/project/src/main.rs");
  });

  it("returns an already-absolute POSIX path unchanged", () => {
    expect(resolveDiffPath("/etc/hosts", "/Users/dev/project")).toBe("/etc/hosts");
  });

  it("returns an already-absolute Windows path unchanged", () => {
    expect(resolveDiffPath("C:\\Users\\dev\\file.rs", "C:\\Users\\dev\\project")).toBe("C:\\Users\\dev\\file.rs");
  });

  it("returns falsy input unchanged instead of throwing", () => {
    expect(resolveDiffPath("", "/Users/dev/project")).toBe("");
    expect(resolveDiffPath(null, "/Users/dev/project")).toBeNull();
  });
});
