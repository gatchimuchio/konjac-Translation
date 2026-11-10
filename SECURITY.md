# Security and Privacy Policy — Konjac Translation

**Project status:** a local Rust text-input runtime is present in source, but there is currently **no shipped runtime or supported security patch release**. The local server binds only to loopback, does not call a translation engine, and does not persist or log request text. While U01 is blocked, the browser checks capability without sending the source text to the local service. Source text can be copied to the system clipboard only after an explicit button click; the page never reads clipboard contents. The page-scoped session ID and input version are ephemeral. The browser can store only language tags after explicit opt-in; a clear control removes them. Source text, translated text and session context are not stored. Other modalities and delivery actions are not implemented.

Speech, video, private messages, documents and camera content can be highly sensitive. Product implementation must treat capture, interpretation, translation, storage and delivery as separately authorized operations.

## Mandatory future controls

- Obtain and respect OS permissions, source ownership and participant consent. Recording another app's call, screen or media is not inherently permitted.
- Clearly show when microphone/camera/screen capture is active and when data leave the device. No automatic switch to remote processing without an appropriate user-controlled permission and explanation.
- Prefer the least data necessary per job; establish configurable retention and deletion, isolation, encryption in transit/at rest and verifiable cleanup before deploying storage.
- Do not print raw messages, speech transcripts, tokens or secrets in diagnostics. Audit metadata and decisions with appropriate redaction and retention.
- Treat external text, document contents, OCR, subtitles, meeting/chat data and model output as untrusted data. They cannot grant authority or bypass permissions.
- Explicitly separate display from sending, saving, sharing, uploading or writing to a third-party app; each has its own authorization.
- Handle failures/unknown output as failures or SUSPEND, not fabricated translations.
- Do not work around DRM, platform protections, authentication or application-specific recording restrictions.
- Review third-party code, weights, licenses, update integrity and supply chain before delivery.

## Vulnerability disclosure

Please do **not** disclose exploitable details, credentials, recordings or other private information in public GitHub issues. Contact the maintainer privately through an agreed channel; GitHub private vulnerability reporting may be used **if enabled for this repository**. No private contact address or response guarantee is asserted here.

No vulnerability-handling workflow or production incident service exists yet.

