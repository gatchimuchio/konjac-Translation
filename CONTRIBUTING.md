# Contributing to Konjac Translation

Konjac Translation is currently at the **design-only** stage. There is no supported build or runnable application yet.

## Before changes

1. Read [AGENTS.md](AGENTS.md), [Product specification (JA)](docs/PRODUCT_SPEC.ja.md), [Architecture (JA)](docs/ARCHITECTURE.ja.md), [Engine integration (JA)](docs/ENGINE_INTEGRATION.ja.md), [Security](SECURITY.md), [LICENSE](LICENSE), and [NOTICE](NOTICE).
2. Describe the translation **user experience** being improved, including input source, target users/languages, output form, accessibility, device and permission requirements.
3. State whether a proposal changes media adapters, engine integration, user experience or governance. Do not create a second engine or silently modify MINIDORA-BETA.
4. Record source/project provenance before importing dependencies or adapting third-party material.

## Evidence for changes

Every substantive contribution should contain:

- Purpose, scope, current and expected state, and affected acceptance IDs.
- Architecture boundary and data contract changes, including privacy/consent.
- Implemented vs proposed/unverified elements.
- Tests performed with platform, version, command, input, observations and failures; clearly mark tests not run.
- Resource and third-party attribution impacts; deployment, permissions or model weights if relevant.
- Migration/rollback effect when user data or public interfaces change.

## Review expectations

- Preserve the overarching multi-translation product: text, voice, images, video, subtitles, documents, Web, meetings, chats, mobile, desktop, and API—not just one application mode.
- Prefer small changes with bounded responsibility and reproducible evidence.
- Do not assert performance parity from a source project or an unrelated MINIDORA benchmark.
- Keep original content/provenance accessible; do not erase evidence to make checks pass.
- Do not include private HDS internal theory or protected unpublished mappings.
- Do not publish secrets, conversation recordings, transcripts or personal data in issues or test artifacts.

Japanese is the canonical language for detailed product specifications; the root README is available in English with a Japanese overview. English translations should preserve the same scope.

This public repository does not promise individual support or an acceptance SLA. Changes affecting release, architecture authority, licensing or external integrations require the maintainer's explicit decision.

Author: **がっちむち♂ / gatchimuchio**.

