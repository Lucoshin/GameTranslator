# DeepSeek Adaptive Throughput Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Maximize DeepSeek translation throughput while preserving stability and continuously exposing useful progress.

**Architecture:** Keep the existing bounded worker scheduler, but add shared adaptive admission control using additive increase and multiplicative decrease. Optimize OpenAI-compatible requests for DeepSeek prefix caching and connection reuse, then expose scheduler throughput and ETA through the existing Tauri progress event.

**Tech Stack:** Rust 1.80, reqwest blocking client, Tauri 2, React 19, Vitest.

---

### Task 1: Adaptive concurrency controller

**Files:**
- Create: `crates/translation-core/src/adaptive.rs`
- Modify: `crates/translation-core/src/lib.rs`
- Modify: `crates/translation-core/src/orchestrator.rs`
- Test: `crates/translation-core/tests/orchestration.rs`

1. Add failing tests proving concurrency grows after successful batches and shrinks after rate limits.
2. Run `cargo test -p game-translator-translation-core --test orchestration` and confirm failure.
3. Add a shared controller with minimum, initial and maximum concurrency plus global cooldown.
4. Feed batch success, duration and provider errors into the controller.
5. Run the focused tests and commit.

### Task 2: DeepSeek request path

**Files:**
- Modify: `crates/provider-core/src/openai_compatible.rs`
- Test: `crates/provider-core/tests/provider_contract.rs`

1. Add failing HTTP contract tests for a stable system prefix and `user_id`.
2. Configure reqwest pooling, TCP keep-alive and request headers.
3. Split stable translation instructions into a system message so DeepSeek context caching can reuse the prefix.
4. Run provider tests and commit.

### Task 3: Throughput telemetry

**Files:**
- Modify: `crates/translation-core/src/orchestrator.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src/App.tsx`
- Modify: `apps/desktop/src/features/translation/TranslationProgress.tsx`
- Test: `apps/desktop/src/test/translation-progress.test.tsx`

1. Add failing tests for live concurrency, segments-per-second and ETA rendering.
2. Extend progress snapshots and Tauri events with scheduler metrics.
3. Render current concurrency, throughput and ETA without increasing log volume excessively.
4. Run Rust and frontend tests.

### Task 4: Verification and packaging

1. Run `cargo fmt --all -- --check`.
2. Run `cargo test --workspace`.
3. Run `cargo clippy --workspace --all-targets -- -D warnings`.
4. Run `npm test -- --run` and `npm run build` in `apps/desktop`.
5. Run `npx --yes @tauri-apps/cli@latest build` and verify MSI/NSIS output.
6. Commit the completed implementation.
