# AGENTS.md — Konjac Translation Development Contract

## 0. Canonical subject

Product: **Konjac Translation / こんにゃく翻訳**. Purpose: remove language friction across text, speech, images, documents, webpages, video, subtitles, meetings, calls and integrated application workflows.

Do not reinterpret this repository as a generic MINIDORA AI shell, speech-to-text-only app, phone interpreter-only app, or isolated subtitle tool. The source of truth for product intent is [Product specification](docs/PRODUCT_SPEC.ja.md); implementation boundaries are [Architecture](docs/ARCHITECTURE.ja.md).

## 1. Source of truth and change discipline

1. Read this file and the affected specifications before work; inspect current Git history, files, tests and external sources. Preserve authored constraints and existing behavior.
2. Separate **observed source facts**, **design proposals**, **implemented functionality**, **test evidence**, and **unknown/unverified**. Do not assert readiness from a diagram, placeholder, model benchmark or smoke test.
3. Keep an append-only dated rationale for significant changes in [docs/DECISIONS.md](docs/DECISIONS.md). Correct conclusions by identifying the evidence and changed scope; do not rewrite historical reasons.
4. Make the smallest coherent change. Avoid speculative frameworks, duplicate validators, defensive wrappers or hidden workarounds without an observed failure and justification.
5. Do not silently change licenses, public contracts, data formats, engine versions, architectural authority, third-party pins, or release criteria.

## 2. MINIDORA engine ownership (non-negotiable)

- [MINIDORA-BETA](https://github.com/gatchimuchio/MINIDORA-BETA) is the selected engine. Respect its frozen beta version and its `AGENTS.md` / `PUBLICATION_BOUNDARY.md`.
- Preserve the engine's **input machine → core → output machine** and capability inheritance. This repository adds **external translation product functionality only**.
- Source adapters and modality processors may capture media, run ASR/OCR/VAD, extract document structure and supply metadata. They cannot independently assume MINIDORA's meaning decision, learning, terminal adoption or action authority.
- Output adapters may render subtitles, speech, overlays, translations and deliver artifacts. They cannot create a competing meaning/adoption loop.
- No claim that MINIDORA currently performs arbitrary multilingual translation unless verified by actual versioned end-to-end tests.
- Unavailable or ambiguous capabilities: return a specific unsupported/SUSPEND/error outcome with evidence. **Never silently substitute another translation engine** and call the result MINIDORA.

## 3. User experience and extension boundaries

- Design the entire multi-translation product scope from the outset, then implement and validate in tracked slices; do not drop intended modes because a current implementation is incomplete.
- Preserve source/target language IDs, speaker identity, time alignment, document layout references, original content, revision and provenance where applicable.
- Use shared request/event/output contracts across Windows, browser, mobile, services, and approved host-app connectors. Platform-dependent capture remains isolated.
- Do not capture, store, upload, transmit, or inject text/audio/video without the required OS permissions, source rights and explicit user-governed authorization. Local vs remote processing is observable.
- Treat private communications and transcripts as sensitive. Redact logs; establish retention/deletion, isolation and encryption requirements before deployment.

## 4. Third-party sources and attribution

RTranslator (Apache-2.0), Haas-ASR (Apache-2.0), speech.cpp (MIT) are **candidates for analysis and selective lawful reuse**, not a license to transplant all code, model weights, icons or datasets. Record the exact commit, copied paths, notices, dependency terms, models, assets and modifications before import. Do not claim third-party benchmarks as Konjac performance. See [Source review](docs/SOURCE_REVIEW.ja.md) and [NOTICE](NOTICE).

Keep private HDS/engine theory and protected mappings out of this public repository. Publish externally observable interfaces and behavior, not unpublished internal theory.

## 5. Verification and reporting

- A task is not done until requirements, affected interface contracts, negative cases, security, permission boundaries, regression and target runtime evidence are checked.
- Evaluate **quality, latency, fidelity, speaker/timing continuity, output format, error handling, data privacy, portability and usability** per feature, with evidence anchored to version, date and environment.
- Explicitly distinguish `PASS`, `FAIL` and `NOT RUN` / `UNVERIFIED`. No fake or test-only production path. A local run is not Windows-installed/mobile/browser proof.
- Preserve original artifacts and prior outcomes; changes require provenance and rollback path.
- No new release, external publication, third-party integration, device action or irreversible cleanup without user authorization.
- The current repository is **documentation-only**. Do not invent commands, build results, CI runs or executable products until they exist.

## 6. Deliverables and language

- Keep top-level README usable in English and `README.ja.md` usable in Japanese. Detailed Japanese specifications are the initial source of truth; English translations must not silently change meaning.
- For a substantive task, report: changed files, why, observed proof, tests run/not run, limitations and next acceptance gate.
- Preserve author attribution to **がっちむち♂ / gatchimuchio**.

## 7. Precedence

User's current explicit instruction and applicable safety/system rules > repository instructions > dated project design decisions > third-party examples. If a conflict cannot be resolved without changing the purpose/authority boundary, stop and ask for a design decision.

