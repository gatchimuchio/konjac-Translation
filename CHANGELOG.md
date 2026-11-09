# Changelog — Konjac Translation

## Unreleased — 2026-11-04

### Documentation / project initialization

- Established the product as a multi-translation platform with the goal of a language-free user experience.
- Defined MINIDORA-BETA as the selected, independently versioned execution engine; no second translation/adoption engine in outer adapters.
- Documented extended translation scope: text, live speech, subtitles, video, documents, Web, OCR/camera, chat, meetings, mobile, desktop and API.
- Added product requirements, architecture, engine boundary, source analysis, delivery plan, acceptance criteria, and append-only decisions.
- Added English/Japanese README, agent/contribution/security guidelines, license scope and third-party attribution.

### Implementation status

- **No application implementation, CI/test execution, model import, runtime beta, or release has been performed in this initialization.**

## Unreleased — 2026-11-09

### Architecture and implementation contract

- Audited the live MINIDORA-BETA `0.6.0b1` public API and controlled wiring example; retained the real translation path as BLOCKED because no public translation operation or language-pair contract was confirmed.
- Recorded independent MINIDORA successor request MCR-001 in Konjac; it remains unsubmitted and does not alter the frozen beta.
- Selected RTranslator 3.0 at `ae96b008e31323ba8b75f94e8b5d4c29b8c1307e` as the Android external platform foundation. No third-party source, models, or assets were imported.
- Added a Draft 2020-12 cross-platform JSON contract, an OpenAPI 3.1 transport contract, a fail-closed profile for audited MINIDORA versions, and thirteen QC tests for provenance, consent, engine evidence, unsupported outcomes, delivery, API references, and unknown-version handling.
- Replaced feature-by-feature completion gates with the nine-stage coordinated whole-system construction, wiring, verification, target QA, and Owner acceptance sequence.

### Verification

- Konjac contract/API/profile suite: 13 passed (Python 3.12, JSON Schema and OpenAPI validators).
- MINIDORA-BETA source at `fe5822de63644bdaea9208a98046f71684dffae2`: isolated build/install succeeded; controlled wiring example succeeded; published scoped regression command ran 313 tests, all passed. Those results do not demonstrate translation capability.
- No Konjac application, service, persistent store, platform adapter, or end-to-end translation path is implemented.

### Owner-provided construction baseline

- Preserved the uploaded three-point product specification, G0-G8 construction plan, and Codex order verbatim with hashes under `docs/implementation-baseline/`.
- Expanded the derived acceptance matrix to E01-E06 and all U01-U12 × E01-E06 applicability cells. This is scope coverage, not implementation or QA evidence.
- Recorded the supplied Rust common-core direction and Tauri 2 comparison candidate. Installed Rust 1.99.1; no Rust product code or cross-target build is complete.
- Added a logical-model document that distinguishes product contract identity from JSON Schema/OpenAPI projections and lists known projection gaps.
- Audited the specified Haas-ASR and speech.cpp source commits read-only; no code, weights, datasets, or model assets were imported.
- Implemented the first Rust common-core slice: typed logical records, validated BCP 47 tags, scoped consent grant/revocation evaluation, and a fail-closed EnginePort contract. No engine adapter or product execution route was added.
- Added typed TranslationResult/TranslationEvent records, request/result revision binding, full source/target result coverage checks, and ordered event-page validation. Persistence and atomic sequence allocation remain unimplemented.
- Pinned Rust 1.99.1 and Cargo dependencies; recorded locked crate licenses. Language tags now enforce BCP 47 well-formed structure, extension ordering, private use, grandfathered forms, duplicate variant/extension rejection and canonical casing; registry membership and engine support remain separate. Added a closed execution permit bound to the exact language pair, context requirement, and batch/streaming mode, with cross-checks for active session and input revisions, SourceAsset/Span references, engine evidence, and result coverage. Guarded execution checks capture and local/remote processing grants against each source asset before engine capability inspection, and stops for denied capabilities, expired deadlines, or recorded cancellations; request timestamps must follow creation time. Delivery preflight separately checks an authorized artifact's active Deliver grant and binds the permit to the exact artifact for point-of-use revalidation. A process-local idempotency ledger atomically reserves exact requests per session and rejects key reuse with a different body or revision. A process-local event cursor validates consecutive sequences and terminal/delivery ordering across pages. The JSON event projection now accepts the Rust `delivery_artifact_id` field and requires the variant/result/artifact references associated with each event type. Consent evaluation rejects duplicate grant IDs. Authentication service, durable consent/idempotency/events, delivery adapter and running-engine cancellation are still unimplemented. Current Linux checks pass: Rust 32 unit tests, Python 17 contract/profile tests, formatting, and clippy.
- Reconciled the active delivery plan, architecture, acceptance and onboarding workflow with the measured implementation state. G2/G3/G4 and whole-system QC/QA remain open.

