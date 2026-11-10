//! Contracts and orchestration for the Konjac translation entry point.
#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const CONTRACT_VERSION: &str = "konjac.translation.v1";
pub const SELECTED_ENGINE_VERSION: &str = "0.6.0b1";
pub const AUDITED_RUST_CANDIDATE_VERSION: &str = "0.7.0-alpha.8";
pub const AUDITED_RUST_CANDIDATE_COMMIT: &str = "a55402b86090109117ffc68bfc6abe85b2c86012";
pub const MAX_SOURCE_BYTES: usize = 128 * 1024;
pub const MAX_TARGET_LANGUAGES: usize = 16;
const ENGINE_BLOCK_REASON: &str = "A supported MINIDORA translation capability has not been verified and connected. The engine was not called and no translation was generated.";
static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Text,
    Microphone,
    RecordedAudio,
    PhoneConversation,
    DesktopAudio,
    Meeting,
    Video,
    Web,
    Document,
    Image,
    Camera,
    Chat,
    Api,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranslationSubmission {
    pub contract_version: String,
    pub session_id: String,
    pub input_version: u64,
    pub source_kind: SourceKind,
    #[serde(default)]
    pub source_text: Option<String>,
    #[serde(default)]
    pub source_language_hint: Option<String>,
    pub target_languages: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TranslationResponse {
    pub contract_version: &'static str,
    pub request_id: String,
    pub session_id: String,
    pub input_version: u64,
    pub source_kind: SourceKind,
    pub source_language_hint: Option<String>,
    pub status: RequestStatus,
    pub reason_code: &'static str,
    pub reason: &'static str,
    pub target_results: Vec<TargetResult>,
    pub variants: Vec<TranslationVariant>,
    pub delivery_status: &'static str,
    pub engine: EngineEvidence,
    pub retention: &'static str,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestStatus {
    BlockedEngineCapability,
    UnsupportedAdapter,
}

#[derive(Clone, Debug, Serialize)]
pub struct TargetResult {
    pub target_language: String,
    pub status: RequestStatus,
    pub reason_code: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct TranslationVariant {
    pub target_language: String,
    pub status: &'static str,
    pub text: Option<String>,
    pub source_span_id: Option<String>,
    pub result_version: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EngineEvidence {
    pub status: &'static str,
    pub engine: &'static str,
    pub selected_release: &'static str,
    pub audited_rust_candidate: &'static str,
    pub candidate_source_commit: &'static str,
    pub reason: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct ModeCapability {
    pub requirement_id: &'static str,
    pub experience: &'static str,
    pub input_adapter: &'static str,
    pub engine: &'static str,
    /// The browser sends source text only when U01 reports `TRANSLATION_ACCEPTED`.
    pub current_status: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValidationError {
    pub code: &'static str,
    pub message: &'static str,
}

impl TranslationSubmission {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.contract_version != CONTRACT_VERSION {
            return Err(ValidationError {
                code: "UNSUPPORTED_CONTRACT_VERSION",
                message: "contract_version is unsupported",
            });
        }
        if !valid_identity(&self.session_id) {
            return Err(ValidationError {
                code: "INVALID_SESSION_ID",
                message: "session_id must be 1 to 128 ASCII letters, digits, or . _ : -",
            });
        }
        if self.input_version == 0 {
            return Err(ValidationError {
                code: "INVALID_INPUT_VERSION",
                message: "input_version must be greater than zero",
            });
        }
        if self.target_languages.is_empty() || self.target_languages.len() > MAX_TARGET_LANGUAGES {
            return Err(ValidationError {
                code: "INVALID_TARGET_COUNT",
                message: "target_languages must contain 1 to 16 language tags",
            });
        }
        for (i, tag) in self.target_languages.iter().enumerate() {
            if !valid_language_tag(tag) {
                return Err(ValidationError {
                    code: "INVALID_LANGUAGE_TAG",
                    message: "language tags must use bounded ASCII BCP 47 style subtags",
                });
            }
            if self.target_languages[..i]
                .iter()
                .any(|previous| previous.eq_ignore_ascii_case(tag))
            {
                return Err(ValidationError {
                    code: "DUPLICATE_TARGET_LANGUAGE",
                    message: "target_languages must be unique",
                });
            }
        }
        if self
            .source_language_hint
            .as_ref()
            .is_some_and(|tag| !valid_language_tag(tag))
        {
            return Err(ValidationError {
                code: "INVALID_SOURCE_LANGUAGE_HINT",
                message: "source_language_hint must be a valid bounded language tag",
            });
        }
        match (self.source_kind, &self.source_text) {
            (SourceKind::Text, Some(text)) if !text.trim().is_empty() => {
                if text.len() > MAX_SOURCE_BYTES {
                    return Err(ValidationError {
                        code: "SOURCE_TOO_LARGE",
                        message: "source_text exceeds the 131072-byte local request limit",
                    });
                }
            }
            (SourceKind::Text, _) => {
                return Err(ValidationError {
                    code: "EMPTY_SOURCE_TEXT",
                    message: "text requests require nonempty source_text",
                });
            }
            (_, Some(_)) => {
                return Err(ValidationError {
                    code: "SOURCE_KIND_CONTENT_MISMATCH",
                    message: "non-text sources cannot be represented as source_text",
                });
            }
            (_, None) => {}
        }
        Ok(())
    }
}

pub fn execute(submission: TranslationSubmission) -> Result<TranslationResponse, ValidationError> {
    submission.validate()?;
    let request_id = next_request_id();
    let text_adapter_available = submission.source_kind == SourceKind::Text;
    let (status, reason_code, reason, engine_status) = if text_adapter_available {
        (
            RequestStatus::BlockedEngineCapability,
            "TRANSLATION_API_UNAVAILABLE",
            ENGINE_BLOCK_REASON,
            "UNVERIFIED_NOT_CALLED",
        )
    } else {
        (
            RequestStatus::UnsupportedAdapter,
            "INPUT_ADAPTER_UNAVAILABLE",
            "This input mode has no installed Konjac adapter. The source was not acquired.",
            "NOT_CALLED",
        )
    };
    let target_results = submission
        .target_languages
        .iter()
        .map(|target_language| TargetResult {
            target_language: target_language.clone(),
            status,
            reason_code,
        })
        .collect();
    Ok(TranslationResponse {
        contract_version: CONTRACT_VERSION,
        request_id,
        session_id: submission.session_id,
        input_version: submission.input_version,
        source_kind: submission.source_kind,
        source_language_hint: submission.source_language_hint,
        status,
        reason_code,
        reason,
        target_results,
        variants: Vec::new(),
        delivery_status: "NOT_REQUESTED",
        engine: EngineEvidence {
            status: engine_status,
            engine: "MINIDORA-BETA",
            selected_release: SELECTED_ENGINE_VERSION,
            audited_rust_candidate: AUDITED_RUST_CANDIDATE_VERSION,
            candidate_source_commit: AUDITED_RUST_CANDIDATE_COMMIT,
            reason: if text_adapter_available {
                ENGINE_BLOCK_REASON
            } else {
                "Translation was not called because input acquisition is unsupported."
            },
        },
        retention: "EPHEMERAL_REQUEST_ONLY",
    })
}

pub fn mode_capabilities() -> Vec<ModeCapability> {
    vec![
        mode(
            "U01",
            "Text translation",
            "TEXT_ENTRY_IMPLEMENTED",
            "UNVERIFIED_NOT_CALLED",
            "BLOCKED_ENGINE_CAPABILITY",
        ),
        mode(
            "U02",
            "Two-way phone conversation",
            "UNSUPPORTED",
            "NOT_CALLED",
            "UNSUPPORTED_INPUT_ADAPTER",
        ),
        mode(
            "U03",
            "Audio, recording, and transcript",
            "UNSUPPORTED",
            "NOT_CALLED",
            "UNSUPPORTED_INPUT_ADAPTER",
        ),
        mode(
            "U04",
            "Permitted desktop audio captions",
            "UNSUPPORTED",
            "NOT_CALLED",
            "UNSUPPORTED_INPUT_ADAPTER",
        ),
        mode(
            "U05",
            "Meetings and calls",
            "UNSUPPORTED",
            "NOT_CALLED",
            "UNSUPPORTED_INPUT_ADAPTER",
        ),
        mode(
            "U06",
            "Video and stream subtitles",
            "UNSUPPORTED",
            "NOT_CALLED",
            "UNSUPPORTED_INPUT_ADAPTER",
        ),
        mode(
            "U07",
            "Web content",
            "UNSUPPORTED",
            "NOT_CALLED",
            "UNSUPPORTED_INPUT_ADAPTER",
        ),
        mode(
            "U08",
            "PDF and Office documents",
            "UNSUPPORTED",
            "NOT_CALLED",
            "UNSUPPORTED_INPUT_ADAPTER",
        ),
        mode(
            "U09",
            "Camera and images",
            "UNSUPPORTED",
            "NOT_CALLED",
            "UNSUPPORTED_INPUT_ADAPTER",
        ),
        mode(
            "U10",
            "Chat, social, and email",
            "UNSUPPORTED",
            "NOT_CALLED",
            "UNSUPPORTED_INPUT_ADAPTER",
        ),
        mode(
            "U11",
            "Cross-media language and context continuity",
            "NOT_IMPLEMENTED",
            "NOT_CALLED",
            "NOT_IMPLEMENTED",
        ),
        mode(
            "U12",
            "Authenticated API and SDK jobs",
            "LOCAL_CONTRACT_ONLY",
            "UNVERIFIED_NOT_CALLED",
            "BLOCKED_ENGINE_CAPABILITY",
        ),
    ]
}

fn mode(
    requirement_id: &'static str,
    experience: &'static str,
    input_adapter: &'static str,
    engine: &'static str,
    current_status: &'static str,
) -> ModeCapability {
    ModeCapability {
        requirement_id,
        experience,
        input_adapter,
        engine,
        current_status,
    }
}

fn next_request_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let sequence = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
    format!("k-{millis:x}-{:x}-{sequence:x}", std::process::id())
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
}

fn valid_language_tag(value: &str) -> bool {
    if value.is_empty() || value.len() > 63 {
        return false;
    }
    let mut parts = value.split('-');
    let Some(language) = parts.next() else {
        return false;
    };
    (language.len() == 1 || (2..=8).contains(&language.len()))
        && language.bytes().all(|b| b.is_ascii_alphabetic())
        && parts.all(|part| {
            !part.is_empty() && part.len() <= 8 && part.bytes().all(|b| b.is_ascii_alphanumeric())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn submission(
        source_kind: SourceKind,
        source_text: Option<&str>,
        targets: &[&str],
    ) -> TranslationSubmission {
        TranslationSubmission {
            contract_version: CONTRACT_VERSION.into(),
            session_id: "session-1".into(),
            input_version: 1,
            source_kind,
            source_text: source_text.map(str::to_owned),
            source_language_hint: Some("ja".into()),
            target_languages: targets.iter().map(|tag| (*tag).into()).collect(),
        }
    }

    #[test]
    fn text_request_fails_closed_without_fabricating_a_variant() {
        let result = execute(submission(
            SourceKind::Text,
            Some("こんにちは"),
            &["en", "fr"],
        ))
        .expect("valid request");
        assert_eq!(result.status, RequestStatus::BlockedEngineCapability);
        assert_eq!(result.target_results.len(), 2);
        assert!(result.variants.is_empty());
        assert_eq!(result.delivery_status, "NOT_REQUESTED");
        assert_eq!(result.retention, "EPHEMERAL_REQUEST_ONLY");
        assert_eq!(result.engine.selected_release, "0.6.0b1");
        assert_eq!(result.engine.audited_rust_candidate, "0.7.0-alpha.8");
        assert_eq!(
            result.engine.candidate_source_commit,
            "a55402b86090109117ffc68bfc6abe85b2c86012"
        );
    }

    #[test]
    fn unimplemented_media_mode_is_rejected_before_engine_execution() {
        let result =
            execute(submission(SourceKind::Camera, None, &["en"])).expect("valid typed request");
        assert_eq!(result.status, RequestStatus::UnsupportedAdapter);
        assert_eq!(result.engine.status, "NOT_CALLED");
        assert_eq!(result.reason_code, "INPUT_ADAPTER_UNAVAILABLE");
    }

    #[test]
    fn rejects_duplicate_targets_and_mismatched_media_content() {
        let duplicate = submission(SourceKind::Text, Some("hello"), &["en", "EN"]);
        assert_eq!(
            duplicate.validate().unwrap_err().code,
            "DUPLICATE_TARGET_LANGUAGE"
        );
        let mismatch = submission(SourceKind::Video, Some("hello"), &["ja"]);
        assert_eq!(
            mismatch.validate().unwrap_err().code,
            "SOURCE_KIND_CONTENT_MISMATCH"
        );
    }

    #[test]
    fn full_acceptance_mode_population_remains_visible() {
        let modes = mode_capabilities();
        assert_eq!(modes.len(), 12);
        assert_eq!(modes.first().map(|m| m.requirement_id), Some("U01"));
        assert_eq!(modes.last().map(|m| m.requirement_id), Some("U12"));
    }
}
