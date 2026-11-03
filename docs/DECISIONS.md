# Architectural decision log — Konjac Translation

Historical decisions are **append-only**. Revisions must cite changed evidence and supersede an earlier decision explicitly; do not silently edit the meaning of an old entry.

| Date (JST) | Decision | Rationale and scope | Status |
|---|---|---|---|
| 2026-10-21 | Name `konjacTranslation` / こんにゃく翻訳. | The purpose is to talk across language barriers, not require the user to study another language. | Product intent |
| 2026-11-02 | Use MINIDORA as the selected drive engine and study RTranslator/Haas-ASR surrounding machinery. | Separate the engine's input/core/output from OS audio, UI, transport, TTS, ASR, segmentation, model resources. | Architecture direction; integration not executed |
| 2026-11-03 | Reject a general MINIDORA-purpose shell interpretation. | **Konjac is a multi-translation product**. Generic AI execution is not the product subject. | Binding scope correction |
| 2026-11-03 | Extend translation beyond speech-to-text and phone simultaneous interpretation. | Include text, transcription, speech, video, subtitles/dubbing, images/OCR, documents, Web, chat, calls, meetings, APIs and integrated workflows; ideal user experience is language-free. | Binding product scope |
| 2026-11-04 | Create repository-level documentation and governance baseline. | Preserve product scope, MINIDORA ownership, source traceability, licensing distinctions and honest readiness reporting. | This documentation initialization only |
| 2026-11-04 | Adopt the owner's existing artifact-class license split for original Konjac work. | Apache-2.0 for executable/software artifacts; CC BY 4.0 for explanatory documentation; per-file/third-party exceptions take priority. This matches the currently published MINIDORA-BETA and BLUE-TANUKI licensing approach. | Policy baseline |

## Open design decisions (not implicitly settled)

- Installed Windows app UI technology and exact device/OS version support.
- Choice, benchmarks and permitted redistribution of ASR/OCR/TTS models.
- Local/remote processing policy by use case, authenticated endpoints and storage.
- MINIDORA API extensions or independent version if actual multilingual translation is unavailable.
- Service availability, commercial distribution, pricing and public launch timing.

A plan or sample implementation may explore these choices, but must not mark them decided or implemented without an explicit evidence/decision entry.

