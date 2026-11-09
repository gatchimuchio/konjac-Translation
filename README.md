# Konjac Translation — こんにゃく翻訳

**Break language barriers across everyday experiences.**  
[日本語 README](README.ja.md) · [Product specification (JA)](docs/PRODUCT_SPEC.ja.md) · [Architecture (JA)](docs/ARCHITECTURE.ja.md)

Konjac Translation (project name: `konjacTranslation`, nickname: **こんにゃく翻訳**) is a proposed **multimodal, multi-surface translation platform**. Its purpose is for people to communicate, read, watch, listen, create, and collaborate without having to operate different languages manually.

This is a **translation product**, not a generic AI-agent platform, and not merely a phone interpreter, speech-to-text tool, or subtitle generator.

> **Status — 2026-11-09.** A Rust logical-contract core, consent evaluator, and fail-closed MINIDORA `EnginePort` boundary are implemented and unit-tested. The engine runtime adapter, UI, service, persistence, media adapters, and end-to-end routes are **not implemented**. No end-to-end translation capability is demonstrated.

## One translation experience, many sources and destinations

| Experience | Inbound | Outbound / intended experience |
|---|---|---|
| Translate like a general translation service | Typed text, pasted text, shared text | Translated text, alternatives, copy/share |
| Speak across languages | Microphone, audio recording, two-way conversation | Live translation, readable text, optional translated speech |
| Follow conversations and meetings | Live audio, desktop/system audio where permitted, calls/conferences | Captions, multilingual subtitles, transcripts, meeting notes |
| Watch and listen | Video, streams, recorded audio | Time-aligned translated subtitles, transcription, optional dubbing |
| Read materials | Websites, PDF, Office documents, e-books | Translation with navigation, structure and layout preserved where feasible |
| Understand the physical world | Camera, photos, screenshots | OCR-assisted translation with source/translated overlay |
| Communicate in apps | Messages, chats, browser/desktop/mobile share flows | Translation in the appropriate source context without repeated copy/paste |
| Build integrations | Documented API/SDK and approved connectors | The same translation pipeline, permissions, and output contracts |

**User experience principle:** a user chooses language preferences once, and the platform carries source/target language, speaker, terminology, context, timestamps and provenance between views and devices. Outputs can be rendered as text, captions, audio, translated documents or overlays, without forcing the user to rebuild the translation job each time. Automatic capture/transmission remains permission-bound.

## Engine boundary

Konjac's translation execution engine is [**MINIDORA-BETA**](https://github.com/gatchimuchio/MINIDORA-BETA), which retains its existing **input machine → MINIDORA core → output machine** architecture. Konjac owns the **external translation product**: capture/connectors, audio transcription, OCR/text extraction, media alignment, layouts, delivery, UI, sessions, access controls and observability.

**No second inference/adoption engine** may be built into Konjac's outer layers. Perception/transcription and presentation/synthesis may use specialized components when separately licensed and verified; these do **not** establish that MINIDORA can currently translate arbitrary languages. Missing engine capability must be reported and stopped, never simulated as success.

See [Architecture](docs/ARCHITECTURE.ja.md) and [Engine integration contract](docs/ENGINE_INTEGRATION.ja.md).

The current owner-provided product specification, whole-system construction plan, and Codex order are preserved under [`docs/implementation-baseline/`](docs/implementation-baseline/README.ja.md). They define all six target environments and the G0–G8 construction sequence; the repository status and implementation evidence are tracked separately.

## External platform foundation and references

- [RTranslator 3.0](https://github.com/niedev/RTranslator/tree/ae96b008e31323ba8b75f94e8b5d4c29b8c1307e): selected as the Android external platform foundation for audio/service lifecycle, conversation UI, Bluetooth, system TTS, permissions, and generic resource management. No source is imported yet; its translation engine and translation models are excluded. See [adoption boundary](docs/SOURCE_REVIEW.ja.md).
- [Haas-ASR](https://github.com/third-intelligence/Haas-ASR): Japanese speech recognition and demonstration patterns for chunking, provisional/final captions, timing.
- [speech.cpp](https://github.com/nyosegawa/speech.cpp): typed native speech API and transcription/synthesis/server boundaries.

Haas-ASR and speech.cpp remain analysis references. RTranslator source integration is not complete. [Source analysis](docs/SOURCE_REVIEW.ja.md).

## Shared wire contract

The logical record and state contract is documented in [`contracts/LOGICAL_MODEL.ja.md`](contracts/LOGICAL_MODEL.ja.md) and implemented in [`crates/konjac-core`](crates/konjac-core). [`contracts/konjac-v1.schema.json`](contracts/konjac-v1.schema.json) and [`contracts/openapi-v1.json`](contracts/openapi-v1.json) are current JSON projections; they do not yet fully represent the logical contract. The Rust `EnginePort` is an interface and capability gate, not a MINIDORA runtime adapter. Python remains the audited-version diagnostic and JSON/OpenAPI QC tool. Run the focused checks with:

```sh
python -m pip install -r requirements-dev.lock
python -m unittest discover -s tests -v
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

The 23 Python tests validate JSON/OpenAPI projections and the audited capability profile; the 39 Rust tests validate BCP 47 tag syntax and canonical casing, logical records, consent scope, delivery authorization, atomic idempotency claims, cross-page event lifecycle, active session/input references, output coverage, and fail-closed execution gating bound to language pair, context requirement, and batch/streaming mode. They also verify event-type references, SourceAsset rights/owner/content references, consent for device capture, scalar metadata and reference limits, tagged SourceSpan payload/position records, TranslationRequest null/length/integer boundaries, nested EngineExecution capability evidence and fingerprints, TranslationResult lifecycle conditions and nullable evidence, provisional versus final variant evidence, DeliveryArtifact references and state evidence tied to a final variant and matching capability permit, duplicate consent denial, per-asset capture/local/remote processing grants before engine inspection, request time-window rules, and pre-execution stops for unsupported, cancelled, and expired requests. The idempotency ledger and event cursor are in-memory only. These do not establish registry membership, engine support, an authentication service, durable consent/idempotency/events, in-flight cancellation, actual delivery, engine invocation, target OS support, or end-to-end translation. MINIDORA-BETA `0.6.0b1` does not expose a confirmed translation operation. Rust 1.99.1 is pinned for the common core; Tauri 2 remains a comparison candidate.

## Project navigation

| Document | Purpose |
|---|---|
| [Japanese overview](README.ja.md) | User-facing purpose and product scope |
| [Product specification](docs/PRODUCT_SPEC.ja.md) | Translation experiences, intended capabilities and boundaries |
| [Architecture](docs/ARCHITECTURE.ja.md) | Konjac modules, data flows and extension points |
| [Engine integration](docs/ENGINE_INTEGRATION.ja.md) | MINIDORA-BETA integration, verified/unverified capabilities |
| [Source review](docs/SOURCE_REVIEW.ja.md) | RTranslator / Haas-ASR / speech.cpp extraction and limitations |
| [Delivery plan](docs/DELIVERY_PLAN.ja.md) | Order of implementation and explicit readiness gates |
| [Acceptance criteria](docs/ACCEPTANCE.ja.md) | Measurable product, security and quality evidence |
| [Decision log](docs/DECISIONS.md) | Dated, auditable design decisions |
| [Agent instructions](AGENTS.md), [Contributing](CONTRIBUTING.md), [Security](SECURITY.md) | Development and governance |
| [Cloud development setup (JA)](docs/CLOUD_ENVIRONMENT_SETUP.ja.md) | Reproducible current Rust/Python checks and configuration status |
| [LICENSE](LICENSE), [NOTICE](NOTICE), [CHANGELOG](CHANGELOG.md) | Attribution, licensing, history |

**Repository policy:** Software is Apache-2.0; original explanatory documentation is CC BY 4.0, subject to file-level exceptions and third-party rights. See [LICENSE](LICENSE). Attribution: **がっちむち♂ / gatchimuchio**.

