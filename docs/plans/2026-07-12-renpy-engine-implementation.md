# Ren'Py Engine Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a safe Ren'Py adapter that extracts compiled-game dialogue through the bundled runtime and exports standard translation patches.

**Architecture:** Keep engine-specific detection, extraction, placeholder rules, and writeback in `engine-renpy`. Route project operations through an `app-core` engine registry so the desktop does not branch on file formats.

**Tech Stack:** Rust 2021, Tauri 2, Ren'Py command-line translation generator, React/TypeScript.

---

### Task 1: Extend engine-neutral contracts

Write failing tests for `EngineKind::RenPy`, engine display names, and a registry that selects RPG Maker or Ren'Py. Implement the smallest project metadata changes and keep existing RPG Maker tests green.

### Task 2: Detect and parse Ren'Py projects

Create `crates/engine-renpy`. Test detection fixtures and parsing of dialogue plus `old/new` template blocks before implementing detection and parser modules.

### Task 3: Generate templates safely

Test the command plan and cleanup guard using a fake executable fixture. Implement unique probe files, official command invocation, template copying, and unconditional cleanup.

### Task 4: Export Ren'Py patches

Test placeholder preservation and template target replacement. Export only `game/tl/<language>/*.rpy` into a separate patch directory.

### Task 5: Route desktop use cases through the registry

Replace direct RPG Maker calls in Tauri commands with `app-core` scan/extract/export services. Add command tests for both engines and update the UI engine label.

### Task 6: Verify the real game and release build

Scan and extract the supplied Mayfly directory, verify probe cleanup, run workspace tests and Clippy, then rebuild MSI and NSIS installers.
