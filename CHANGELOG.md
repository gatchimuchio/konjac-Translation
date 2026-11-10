# Changelog — Konjac Translation

## Unreleased — 2026-11-10

### Local text-entry implementation

- Added a Rust 1.99.1 workspace, loopback-only Web UI, and versioned JSON contract for typed text and multiple target languages.
- Added an explicit `BLOCKED_ENGINE_CAPABILITY` response. The selected MINIDORA release is not wired, and the audited successor candidate does not establish accepted unrestricted translation. Requests return no translation variant and are not persisted or sent to a remote service.
- Updated the browser UI to refresh the capability status before submission and keep the source text in the browser until U01 explicitly reports `TRANSLATION_ACCEPTED`; failed status checks also send no source text.
- Added optional, deletable browser-local language preferences for U11. Only the source-language hint and target-language tags are stored; source and translated text remain excluded. Cross-media context and device sync are not implemented.
- Added explicit source-text copy and clear controls. Clipboard writes occur only after a button click; the page never reads clipboard contents.
- Added an in-memory page session ID and incrementing input version for text, language-hint and target edits. These values reset when the page is reopened and are sent only through the accepted translation path.
- Refreshed the engine evidence reference to the latest audited Rust alpha 8 source commit without changing the selected frozen Python Beta version.
- Kept the U01–U12 product scope visible. Only text entry and the local request/status route are present; other media adapters, authenticated SDK/API, history, platform clients, translation acceptance, and release remain open.
- Recorded locked Serde dependencies and their package license metadata in `NOTICE`.
- Verification is limited to this source version and the commands/results recorded in [acceptance evidence](docs/ACCEPTANCE.ja.md); it does not establish full product QC, QA, or owner release.

### Agent governance and historical engineering reconciliation

- Replaced the root `AGENTS.md` with a standalone Japanese-base Codex engineering contract. Consolidated common audit, responsibility, change, evidence, privacy, provenance and whole-system construction disciplines without importing other products' implementation authority.
- Recorded the owner's 2026-11-09 Rust provisional construction choice, future AICompiler substitutability, full-system BUILD → QC → QA methodology and physical-confirmation/virtual-QA distinction in the append-only decision log.
- Clarified historical incremental implementation language in the delivery plan, acceptance criteria and Japanese README without reducing the full multi-translation product scope.
- **Documentation-only governance revision:** No executable translation service, model, application, tests, MINIDORA capability proof or product release is created by these changes.

## Unreleased — 2026-11-04

### Documentation / project initialization

- Established the product as a multi-translation platform with the goal of a language-free user experience.
- Defined MINIDORA-BETA as the selected, independently versioned execution engine; no second translation/adoption engine in outer adapters.
- Documented extended translation scope: text, live speech, subtitles, video, documents, Web, OCR/camera, chat, meetings, mobile, desktop and API.
- Added product requirements, architecture, engine boundary, source analysis, delivery plan, acceptance criteria, and append-only decisions.
- Added English/Japanese README, agent/contribution/security guidelines, license scope and third-party attribution.

### Implementation status

- **No application implementation, CI/test execution, model import, runtime beta, or release has been performed in this initialization.**

