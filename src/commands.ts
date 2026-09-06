// Thin, typed wrappers around the Tauri commands in src-tauri/src/commands.rs.
// Full keyboard-shortcut wiring (menus, focus management) is Task 12 — this
// module just proves the invoke() round-trip works, per Task 8.

import { invoke } from "@tauri-apps/api/core";

export interface OpenedDocument {
  path: string;
  text: string;
}

export interface Diagnostic {
  severity: "error" | "warning";
  message: string;
}

export interface PageInfo {
  width_pt: number;
  height_pt: number;
}

export interface CompileResult {
  success: boolean;
  pages: PageInfo[];
  diagnostics: Diagnostic[];
}

export function newDocument(): Promise<void> {
  return invoke("new_document");
}

export function openDocument(): Promise<OpenedDocument | null> {
  return invoke("open_document");
}

export function saveDocument(text: string, saveAs = false): Promise<string> {
  return invoke("save_document", { text, saveAs });
}

export function compileDocument(text: string): Promise<CompileResult> {
  return invoke("compile_document", { text });
}
