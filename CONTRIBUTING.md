# Contributing to Konjac Translation

Konjac Translation has product specifications, a Rust logical-contract core, a platform-neutral JSON projection, and focused QC tests. The translation service and platform applications remain unimplemented. MINIDORA-BETA `0.6.0b1` has no confirmed translation operation in the audited public API.

## Before changes

1. Read [AGENTS.md](AGENTS.md), [Product specification (JA)](docs/PRODUCT_SPEC.ja.md), [Architecture (JA)](docs/ARCHITECTURE.ja.md), [Engine integration (JA)](docs/ENGINE_INTEGRATION.ja.md), [Security](SECURITY.md), [LICENSE](LICENSE), and [NOTICE](NOTICE).
   For managed cloud workflows, also check [cloud setup](docs/CLOUD_ENVIRONMENT_SETUP.ja.md); the guide distinguishes the current machine from saved/published environment configuration.
2. Describe the translation **user experience** being improved, including input source, target users/languages, output form, accessibility, device and permission requirements.
3. State whether a proposal changes media adapters, engine integration, user experience or governance. Do not create a second engine or silently modify MINIDORA-BETA.
4. Record source/project provenance before importing dependencies or adapting third-party material. RTranslator 3.0 `ae96b008e31323ba8b75f94e8b5d4c29b8c1307e` is the selected Android external platform basis; its translation engine and translation models are out of scope.
5. Preserve the full construction sequence in [Delivery Plan](docs/DELIVERY_PLAN.ja.md). Acceptance IDs index coverage; they are not per-feature completion gates.

## Evidence for changes

Every substantive contribution should contain:

- Purpose, scope, current and expected state, and affected acceptance IDs.
- Architecture boundary and data contract changes, including privacy/consent.
- Whole-system construction and wiring state, with each acceptance ID's implementation, wiring, QC, QA, and final acceptance status separated.
- Implemented vs proposed/unverified elements.
- Tests performed with platform, version, command, input, observations and failures; clearly mark tests not run.
- Resource and third-party attribution impacts; deployment, permissions or model weights if relevant.
- Migration/rollback effect when user data or public interfaces change.

## Review expectations

- Preserve the overarching multi-translation product: text, voice, images, video, subtitles, documents, Web, meetings, chats, mobile, desktop, and API—not just one application mode.
- Keep component responsibilities bounded while coordinating their construction and wiring across the full product scope.
- Do not assert performance parity from a source project or an unrelated MINIDORA benchmark.
- Keep original content/provenance accessible; do not erase evidence to make checks pass.
- Do not include private HDS internal theory or protected unpublished mappings.
- Do not publish secrets, conversation recordings, transcripts or personal data in issues or test artifacts.

Japanese is the canonical language for detailed product specifications; the root README is available in English with a Japanese overview. English translations should preserve the same scope.

This public repository does not promise individual support or an acceptance SLA. Changes affecting release, architecture authority, licensing or external integrations require the maintainer's explicit decision.

Author: **がっちむち♂ / gatchimuchio**.

## Contract QC

Install the schema checker and run the platform-neutral contract checks from the repository root:

```sh
python -m pip install -r requirements-dev.lock
python -m unittest discover -s tests -v
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

These checks exercise JSON/OpenAPI projections, capability inspection, logical record invariants, scoped consent, and the closed engine gate. They do not establish engine execution, durable storage, app behavior, target OS support, or product acceptance. Set `RUSTUP_HOME=/workspace/.rustup`, `CARGO_HOME=/workspace/.cargo`, and add `/workspace/.cargo/bin` to `PATH` when the managed shell has not activated the pinned Rust toolchain.

