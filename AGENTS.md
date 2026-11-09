# AGENTS.md — Konjac Translation Development Contract

## 0. Canonical subject

Product: **Konjac Translation / こんにゃく翻訳**. Purpose: remove language friction across text, speech, images, documents, webpages, video, subtitles, meetings, calls and integrated application workflows.

Do not reinterpret this repository as a generic MINIDORA AI shell, speech-to-text-only app, phone interpreter-only app, or isolated subtitle tool. The current owner-provided product and construction baseline is preserved in [01 product specification](docs/implementation-baseline/01_KONJAC_%E7%B5%B1%E5%90%88%E8%A3%BD%E5%93%81%E5%AE%9F%E8%A3%85%E4%BB%95%E6%A7%98%E6%9B%B8_v1_20261109.md), [02 whole-system plan](docs/implementation-baseline/02_KONJAC_%E5%85%A8%E4%BD%93%E6%96%BD%E5%B7%A5%E5%B7%A5%E7%A8%8B%E8%A1%A8_v1_20261109.md), and [03 Codex construction order](docs/implementation-baseline/03_KONJAC_Codex%E5%85%A8%E4%BD%93%E7%B3%BB%E6%96%BD%E5%B7%A5%E6%8C%87%E7%A4%BA%E6%9B%B8_v1_20261109.md). Read all three before product changes. The original files and their hashes are recorded in [the source receipt](docs/implementation-baseline/README.ja.md). The existing [Product specification](docs/PRODUCT_SPEC.ja.md) and [Architecture](docs/ARCHITECTURE.ja.md) are derived repository documents and must stay aligned with that baseline.

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

- Design and construct the entire multi-translation system across its declared scope. A01–A07 and U01–U12 are coverage and acceptance indexes, not per-feature completion gates. Component checks needed for type, safety, build integrity, and whole-system wiring are allowed; do not switch to a repeated pattern of accepting one feature before starting the next.
- Preserve source/target language IDs, speaker identity, time alignment, document layout references, original content, revision and provenance where applicable.
- Use one logical request/event/output contract across Windows, Linux, macOS, Android, iOS, Web, services, and approved host-app connectors. Keep JSON Schema/OpenAPI/Protobuf/ABI as exchange projections; platform-dependent capture remains isolated.
- Use Rust as the selected first choice for the common processing core. Tauri 2 is a desktop/shared-UI comparison candidate, not a completed or accepted platform choice. Python is limited to the MINIDORA inspection and contract-QC tools.
- Do not capture, store, upload, transmit, or inject text/audio/video without the required OS permissions, source rights and explicit user-governed authorization. Local vs remote processing is observable.
- Treat private communications and transcripts as sensitive. Redact logs; establish retention/deletion, isolation and encryption requirements before deployment.

## 4. Third-party sources and attribution

RTranslator 3.0 at `ae96b008e31323ba8b75f94e8b5d4c29b8c1307e` is the selected Android external foundation for service/audio lifecycle, conversation UI, Bluetooth, TTS, permission, and generic resource-management responsibilities. Selection does not mean source integration is complete. Translation engines and translation model assets are excluded. Any imported file must retain Apache-2.0 notices and record exact paths, modifications, dependencies, and separate model/asset terms. Haas-ASR (Apache-2.0) and speech.cpp (MIT) remain analysis candidates. See [Source review](docs/SOURCE_REVIEW.ja.md) and [NOTICE](NOTICE).

Keep private HDS/engine theory and protected mappings out of this public repository. Publish externally observable interfaces and behavior, not unpublished internal theory.

## 5. Verification and reporting

- A task is not done until requirements, affected interface contracts, negative cases, security, permission boundaries, regression and target runtime evidence are checked.
- Evaluate **quality, latency, fidelity, speaker/timing continuity, output format, error handling, data privacy, portability and usability** per feature, with evidence anchored to version, date and environment.
- Explicitly distinguish `PASS`, `FAIL` and `NOT RUN` / `UNVERIFIED`. No fake or test-only production path. A local run is not Windows-installed/mobile/browser proof.
- Preserve original artifacts and prior outcomes; changes require provenance and rollback path.
- No new release, external publication, third-party integration, device action or irreversible cleanup without user authorization.
- The repository contains product/design documentation, a first Rust logical-core implementation, a language-neutral JSON projection, and contract validation tests. It still has no runnable translation product, MINIDORA runtime adapter, service, platform adapter, or CI result unless implemented and verified in the current checkout. Report the scope of each check.

## 6. Deliverables and language

- Keep top-level README usable in English and `README.ja.md` usable in Japanese. Detailed Japanese specifications are the initial source of truth; English translations must not silently change meaning.
- For a substantive task, report: changed files, why, observed proof, tests run/not run, limitations and next acceptance gate.
- For managed cloud setup, use [the current Rust/Python environment procedure](docs/CLOUD_ENVIRONMENT_SETUP.ja.md); it does not imply settings were saved or the environment was published.
- Preserve author attribution to **がっちむち♂ / gatchimuchio**.

## 7. Precedence

User's current explicit instruction and applicable safety/system rules > repository instructions > dated project design decisions > third-party examples. If a conflict cannot be resolved without changing the purpose/authority boundary, stop and ask for a design decision.

