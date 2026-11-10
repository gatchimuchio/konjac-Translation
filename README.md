# Konjac Translation — こんにゃく翻訳

**Break language barriers across everyday experiences.**  
[日本語 README](README.ja.md) · [Product specification (JA)](docs/PRODUCT_SPEC.ja.md) · [Architecture (JA)](docs/ARCHITECTURE.ja.md)

Konjac Translation (project name: `konjacTranslation`, nickname: **こんにゃく翻訳**) is a proposed **multimodal, multi-surface translation platform**. Its purpose is for people to communicate, read, watch, listen, create, and collaborate without having to operate different languages manually.

This is a **translation product**, not a generic AI-agent platform, and not merely a phone interpreter, speech-to-text tool, or subtitle generator.

> **Status — implementation baseline, 2026-11-10.** The repository now contains a runnable Rust local Web interface and JSON endpoint for typed text. It reports a blocked translation request because the selected MINIDORA capability is not connected or accepted for general translation. It does not produce translations. Audio, images, video, documents, Web capture, conversations, persistent history, remote API authentication, Windows/mobile clients, and release packaging remain unimplemented. There is no released beta or product acceptance claim.

Run the local text interface with Rust 1.99.1:

```sh
cargo run --locked -- serve
```

Open `http://127.0.0.1:8787/`. The server binds only to loopback. Its `POST /api/v1/translate` contract preserves source/target language tags and returns a typed `BLOCKED_ENGINE_CAPABILITY` status with no translation variant while the MINIDORA route remains unverified. Requests are handled in memory for one response; raw input is not logged or persisted. See [the current engine evidence](docs/ENGINE_INTEGRATION.ja.md).

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

## Reference projects (study before reuse)

- [RTranslator](https://github.com/niedev/RTranslator): Android voice lifecycle, on-device conversation UI, Bluetooth exchange, TTS, model/resource management.
- [Haas-ASR](https://github.com/third-intelligence/Haas-ASR): Japanese speech recognition and demonstration patterns for chunking, provisional/final captions, timing.
- [speech.cpp](https://github.com/nyosegawa/speech.cpp): typed native speech API and transcription/synthesis/server boundaries.

These are **reference implementations**, not integrated dependencies. Source reuse requires license, attribution, platform, privacy and regression review. [Source analysis](docs/SOURCE_REVIEW.ja.md).

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
| [LICENSE](LICENSE), [NOTICE](NOTICE), [CHANGELOG](CHANGELOG.md) | Attribution, licensing, history |

**Repository policy:** Software is Apache-2.0; original explanatory documentation is CC BY 4.0, subject to file-level exceptions and third-party rights. See [LICENSE](LICENSE). Attribution: **がっちむち♂ / gatchimuchio**.

