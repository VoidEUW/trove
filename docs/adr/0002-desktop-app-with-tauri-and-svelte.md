# 2. Trove is a local-first Tauri 2 desktop app with a Svelte 5 UI

- Status: Accepted
- Date: 2026-10-11
- Origin: chosen in the mockup phase, 2026-10-10
- Issue: #1

## Context

Trove catalogues a household's belongings and shows where they are on a 3D floor plan. It must work
without internet, start on one Mac and stay portable to Windows and Linux. The core is written in
Rust. A phone app that scans QR labels comes later.

## Decision

- The shell is Tauri 2; the UI is Svelte 5 in a SvelteKit single-page app; the 3D view uses
  Threlte (three.js).
- The Rust core runs inside the app. There is no server, no account and no network dependency.
- All data stays in a local data folder (see [8](0008-sqlite-storage.md)).

This rules out Electron, a native SwiftUI app, a Rust-side renderer such as Bevy, CSS-faked 3D and a
hosted web version.

## Consequences

- One codebase for macOS, Windows and Linux; Tauri's mobile targets carry the later scanner app.
- Two toolchains (cargo and npm) in CI and in every setup.
- WebViews differ (WKWebView, WebView2, WebKitGTK); WebGL and fonts must be checked on each.
- Follow-ups: CI build matrix for three systems (the Tauri + SvelteKit scaffold exists).
