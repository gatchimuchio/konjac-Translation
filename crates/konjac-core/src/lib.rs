//! Platform-neutral logical contracts for Konjac Translation.
//!
//! These records define product meaning and safety boundaries. JSON Schema and
//! OpenAPI are exchange projections; neither defines translation capability.
//! This crate contains no translation model or semantic decision engine.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub mod consent;
pub mod delivery;
pub mod engine;
pub mod idempotency;

pub const LOGICAL_CONTRACT_VERSION: &str = "1.0.0";
pub const EXCHANGE_SCHEMA_VERSION: &str = "1.0.0";

/// A validated BCP 47 language identifier, kept separate from display names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct LanguageTag(String);

impl LanguageTag {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

const GRANDFATHERED_LANGUAGE_TAGS: &[&str] = &[
    "art-lojban",
    "cel-gaulish",
    "en-GB-oed",
    "i-ami",
    "i-bnn",
    "i-default",
    "i-enochian",
    "i-hak",
    "i-klingon",
    "i-lux",
    "i-mingo",
    "i-navajo",
    "i-pwn",
    "i-tao",
    "i-tay",
    "i-tsu",
    "no-bok",
    "no-nyn",
    "sgn-BE-FR",
    "sgn-BE-NL",
    "sgn-CH-DE",
    "zh-guoyu",
    "zh-hakka",
    "zh-min",
    "zh-min-nan",
    "zh-xiang",
];

fn is_ascii_alpha(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_alphabetic())
}

fn is_ascii_alphanumeric(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn is_variant_subtag(value: &str) -> bool {
    (5..=8).contains(&value.len()) && is_ascii_alphanumeric(value)
        || value.len() == 4 && value.as_bytes()[0].is_ascii_digit() && is_ascii_alphanumeric(value)
}

impl FromStr for LanguageTag {
    type Err = ContractError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() || value.len() > 63 || !value.is_ascii() {
            return Err(ContractError::InvalidLanguageTag);
        }

        if let Some(grandfathered) = GRANDFATHERED_LANGUAGE_TAGS
            .iter()
            .find(|tag| tag.eq_ignore_ascii_case(value))
        {
            return Ok(Self((*grandfathered).to_owned()));
        }

        let subtags: Vec<_> = value.split('-').collect();
        if subtags.iter().any(|subtag| subtag.is_empty()) {
            return Err(ContractError::InvalidLanguageTag);
        }

        if subtags[0].eq_ignore_ascii_case("x") {
            if subtags.len() < 2
                || subtags[1..]
                    .iter()
                    .any(|subtag| subtag.len() > 8 || !is_ascii_alphanumeric(subtag))
            {
                return Err(ContractError::InvalidLanguageTag);
            }
            return Ok(Self(
                subtags
                    .iter()
                    .map(|subtag| subtag.to_ascii_lowercase())
                    .collect::<Vec<_>>()
                    .join("-"),
            ));
        }

        let primary = subtags[0];
        let primary_length = primary.len();
        if !(2..=8).contains(&primary_length) || !is_ascii_alpha(primary) {
            return Err(ContractError::InvalidLanguageTag);
        }

        let mut canonical = vec![primary.to_ascii_lowercase()];
        let mut index = 1;
        if primary_length <= 3 {
            let mut extlang_count = 0;
            while extlang_count < 3
                && subtags
                    .get(index)
                    .is_some_and(|subtag| subtag.len() == 3 && is_ascii_alpha(subtag))
            {
                canonical.push(subtags[index].to_ascii_lowercase());
                index += 1;
                extlang_count += 1;
            }
        }

        if subtags
            .get(index)
            .is_some_and(|subtag| subtag.len() == 4 && is_ascii_alpha(subtag))
        {
            let script = subtags[index].to_ascii_lowercase();
            let mut chars = script.chars();
            let first = chars
                .next()
                .ok_or(ContractError::InvalidLanguageTag)?
                .to_ascii_uppercase();
            canonical.push(format!("{first}{}", chars.as_str()));
            index += 1;
        }

        if subtags.get(index).is_some_and(|subtag| {
            (subtag.len() == 2 && is_ascii_alpha(subtag))
                || (subtag.len() == 3 && subtag.bytes().all(|byte| byte.is_ascii_digit()))
        }) {
            canonical.push(subtags[index].to_ascii_uppercase());
            index += 1;
        }

        let mut variants = std::collections::BTreeSet::new();
        while subtags
            .get(index)
            .is_some_and(|subtag| is_variant_subtag(subtag))
        {
            let variant = subtags[index].to_ascii_lowercase();
            if !variants.insert(variant.clone()) {
                return Err(ContractError::InvalidLanguageTag);
            }
            canonical.push(variant);
            index += 1;
        }

        let mut extensions = std::collections::BTreeSet::new();
        while subtags.get(index).is_some_and(|subtag| {
            subtag.len() == 1 && is_ascii_alphanumeric(subtag) && !subtag.eq_ignore_ascii_case("x")
        }) {
            let singleton = subtags[index].to_ascii_lowercase();
            if !extensions.insert(singleton.clone()) {
                return Err(ContractError::InvalidLanguageTag);
            }
            canonical.push(singleton);
            index += 1;

            let first_extension_value = index;
            while subtags.get(index).is_some_and(|subtag| {
                (2..=8).contains(&subtag.len()) && is_ascii_alphanumeric(subtag)
            }) {
                canonical.push(subtags[index].to_ascii_lowercase());
                index += 1;
            }
            if index == first_extension_value {
                return Err(ContractError::InvalidLanguageTag);
            }
        }

        if subtags
            .get(index)
            .is_some_and(|subtag| subtag.eq_ignore_ascii_case("x"))
        {
            canonical.push("x".into());
            index += 1;
            let private_use_start = index;
            while subtags
                .get(index)
                .is_some_and(|subtag| subtag.len() <= 8 && is_ascii_alphanumeric(subtag))
            {
                canonical.push(subtags[index].to_ascii_lowercase());
                index += 1;
            }
            if index == private_use_start {
                return Err(ContractError::InvalidLanguageTag);
            }
        }

        if index != subtags.len() {
            return Err(ContractError::InvalidLanguageTag);
        }
        Ok(Self(canonical.join("-")))
    }
}

impl<'de> Deserialize<'de> for LanguageTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

macro_rules! define_record {
    ($name:ident { $($fields:tt)* }) => {
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            /// Version of the product's logical contract, independent of a wire format.
            pub contract_version: String,
            /// Stable record identity; retries must not create false identity matches.
            pub record_id: Uuid,
            /// Creation time in UTC. This is not a media timestamp.
            pub created_at: DateTime<Utc>,
            $($fields)*
        }
    };
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionPolicy {
    Ephemeral,
    Session,
    UserManaged,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingLocation {
    LocalOnly,
    RemoteWithConsent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Text,
    Audio,
    Image,
    Video,
    Document,
    Web,
    Chat,
    Meeting,
    Api,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputOrigin {
    UserInput,
    FilePicker,
    Camera,
    Microphone,
    SystemAudio,
    BrowserSelection,
    ApprovedConnector,
    ApiClient,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentScope {
    Capture,
    ProcessLocal,
    ProcessRemote,
    Store,
    Deliver,
    Share,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentReference {
    pub kind: ReferenceKind,
    pub key: String,
    pub media_type: Option<String>,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
}

impl ContentReference {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.key.is_empty() || self.key.chars().count() > 1024 {
            return Err(ContractError::InvalidReference);
        }
        if self
            .media_type
            .as_deref()
            .is_some_and(|media_type| media_type.is_empty() || media_type.chars().count() > 127)
        {
            return Err(ContractError::InvalidReference);
        }
        validate_optional_sha256(self.sha256.as_deref())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceKind {
    Local,
    Remote,
}

define_record!(SourceAsset {
    pub media_kind: MediaKind,
    pub origin: InputOrigin,
    pub media_type: String,
    pub content_ref: Option<ContentReference>,
    pub original_fingerprint: Option<String>,
    pub owner_ref: Option<ContentReference>,
    pub rights_ref: Option<ContentReference>,
    pub capture_consent_id: Option<Uuid>,
    pub time_basis: Option<String>,
    pub retention: RetentionPolicy,
    pub metadata: BTreeMap<String, serde_json::Value>,
});

impl SourceAsset {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_base(&self.contract_version, &self.media_type)?;
        if self.media_type.chars().count() > 127
            || self
                .metadata
                .values()
                .any(|value| value.is_array() || value.is_object())
        {
            return Err(ContractError::InvalidSourceAsset);
        }
        if matches!(
            self.origin,
            InputOrigin::Camera | InputOrigin::Microphone | InputOrigin::SystemAudio
        ) && self.capture_consent_id.is_none()
        {
            return Err(ContractError::MissingCaptureConsent);
        }
        validate_optional_sha256(self.original_fingerprint.as_deref())?;
        for reference in [
            self.content_ref.as_ref(),
            self.owner_ref.as_ref(),
            self.rights_ref.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            reference.validate()?;
        }
        Ok(())
    }
}

define_record!(ConsentGrant {
    pub principal_id: String,
    pub scope: ConsentScope,
    pub subject_ids: Vec<Uuid>,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub evidence_ref: Option<ContentReference>,
});

impl ConsentGrant {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_base(&self.contract_version, &self.principal_id)?;
        if self.principal_id.chars().count() > 256
            || self.subject_ids.is_empty()
            || self
                .expires_at
                .is_some_and(|expires| expires <= self.granted_at)
        {
            return Err(ContractError::InvalidConsent);
        }
        validate_unique(&self.subject_ids)?;
        if let Some(reference) = &self.evidence_ref {
            reference.validate()?;
        }
        Ok(())
    }
}

define_record!(ConsentRevocation {
    pub grant_id: Uuid,
    pub revoked_by: String,
    pub revoked_at: DateTime<Utc>,
    pub reason_code: String,
});

impl ConsentRevocation {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_base(&self.contract_version, &self.revoked_by)?;
        if self.revoked_by.chars().count() > 256
            || self.reason_code.is_empty()
            || self.reason_code.chars().count() > 128
        {
            return Err(ContractError::InvalidRevocation);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Paused,
    Closed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantRole {
    Owner,
    Speaker,
    Listener,
    Service,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Participant {
    pub participant_id: Uuid,
    pub role: ParticipantRole,
    pub preferred_language: Option<LanguageTag>,
    pub consent_grant_ids: Vec<Uuid>,
}

define_record!(TranslationSession {
    pub revision: u64,
    pub status: SessionStatus,
    pub source_language: Option<LanguageTag>,
    pub target_languages: Vec<LanguageTag>,
    pub participants: Vec<Participant>,
    pub context_refs: Vec<ContentReference>,
    pub terminology_refs: Vec<ContentReference>,
    pub processing_location: ProcessingLocation,
    pub consent_grant_ids: Vec<Uuid>,
    pub retention: RetentionPolicy,
});

impl TranslationSession {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_base(&self.contract_version, "TranslationSession")?;
        if self.revision == 0 || self.target_languages.is_empty() || self.participants.is_empty() {
            return Err(ContractError::InvalidSession);
        }
        validate_unique(&self.target_languages)?;
        validate_unique(&self.consent_grant_ids)?;
        validate_unique(&self.context_refs)?;
        validate_unique(&self.terminology_refs)?;
        validate_unique(
            &self
                .participants
                .iter()
                .map(|participant| participant.participant_id)
                .collect::<Vec<_>>(),
        )?;
        for participant in &self.participants {
            validate_unique(&participant.consent_grant_ids)?;
        }
        if self.processing_location == ProcessingLocation::RemoteWithConsent
            && self.consent_grant_ids.is_empty()
        {
            return Err(ContractError::MissingProcessingConsent);
        }
        for reference in self.context_refs.iter().chain(&self.terminology_refs) {
            reference.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationOrigin {
    Typed,
    Asr,
    Ocr,
    DocumentParser,
    WebExtractor,
    Connector,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SpanPayload {
    Text { text: String },
    Media { reference: ContentReference },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourcePosition {
    AudioTime {
        start_ms: u64,
        end_ms: u64,
    },
    VideoTime {
        start_ms: u64,
        end_ms: u64,
    },
    DocumentAnchor {
        anchor: String,
        page: Option<u32>,
    },
    ImageBounds {
        page: Option<u32>,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
}

impl SourcePosition {
    fn validate(&self) -> Result<(), ContractError> {
        match self {
            Self::AudioTime { start_ms, end_ms } | Self::VideoTime { start_ms, end_ms }
                if start_ms >= end_ms =>
            {
                Err(ContractError::InvalidSourcePosition)
            }
            Self::DocumentAnchor { anchor, page }
                if anchor.is_empty() || anchor.chars().count() > 1024 || page == &Some(0) =>
            {
                Err(ContractError::InvalidSourcePosition)
            }
            Self::ImageBounds {
                page,
                x,
                y,
                width,
                height,
            } if page == &Some(0)
                || ![x, y, width, height]
                    .into_iter()
                    .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
                || width == &0.0
                || height == &0.0
                || x + width > 1.0
                || y + height > 1.0 =>
            {
                Err(ContractError::InvalidSourcePosition)
            }
            _ => Ok(()),
        }
    }
}

define_record!(SourceSpan {
    pub asset_id: Uuid,
    pub session_id: Uuid,
    pub revision: u64,
    pub payload: SpanPayload,
    pub source_language: Option<LanguageTag>,
    pub observation_origin: ObservationOrigin,
    pub confidence: Option<f32>,
    pub speaker_id: Option<Uuid>,
    pub position: Option<SourcePosition>,
    pub provenance_ref: Option<ContentReference>,
});

impl SourceSpan {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_base(&self.contract_version, "SourceSpan")?;
        if self.revision == 0 {
            return Err(ContractError::InvalidSourceSpan);
        }
        if let SpanPayload::Text { text } = &self.payload
            && text.is_empty()
        {
            return Err(ContractError::InvalidSourceSpan);
        }
        if self
            .confidence
            .is_some_and(|confidence| !confidence.is_finite() || !(0.0..=1.0).contains(&confidence))
        {
            return Err(ContractError::InvalidConfidence);
        }
        if let Some(position) = &self.position {
            position.validate()?;
        }
        if let SpanPayload::Media { reference } = &self.payload {
            reference.validate()?;
        }
        if let Some(reference) = &self.provenance_ref {
            reference.validate()?;
        }
        Ok(())
    }

    pub fn validate_against(
        &self,
        asset: &SourceAsset,
        session: &TranslationSession,
    ) -> Result<(), ContractError> {
        self.validate()?;
        asset.validate()?;
        session.validate()?;
        if self.asset_id != asset.record_id || self.session_id != session.record_id {
            return Err(ContractError::SourceAssetReferenceMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestedOutput {
    Text,
    Subtitle,
    Audio,
    Document,
    Overlay,
    Api,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Batch,
    Streaming,
}

define_record!(TranslationRequest {
    pub session_id: Uuid,
    pub session_revision: u64,
    pub revision: u64,
    pub source_span_ids: Vec<Uuid>,
    pub input_revision: u64,
    pub target_languages: Vec<LanguageTag>,
    pub context_refs: Vec<ContentReference>,
    pub terminology_refs: Vec<ContentReference>,
    pub requested_outputs: Vec<RequestedOutput>,
    pub execution_mode: ExecutionMode,
    pub processing_location: ProcessingLocation,
    pub processing_consent_id: Option<Uuid>,
    pub idempotency_key: String,
    pub attempt: u32,
    pub deadline: Option<DateTime<Utc>>,
    pub cancel_requested_at: Option<DateTime<Utc>>,
});

impl TranslationRequest {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_base(&self.contract_version, &self.idempotency_key)?;
        if self.source_span_ids.is_empty()
            || self.target_languages.is_empty()
            || self.requested_outputs.is_empty()
            || self.session_revision == 0
            || self.revision == 0
            || self.input_revision == 0
            || self.attempt == 0
            || !(16..=256).contains(&self.idempotency_key.chars().count())
            || self
                .deadline
                .is_some_and(|deadline| deadline <= self.created_at)
            || self
                .cancel_requested_at
                .is_some_and(|cancelled_at| cancelled_at < self.created_at)
        {
            return Err(ContractError::InvalidRequest);
        }
        validate_unique(&self.source_span_ids)?;
        validate_unique(&self.target_languages)?;
        validate_unique(&self.requested_outputs)?;
        validate_unique(&self.context_refs)?;
        validate_unique(&self.terminology_refs)?;
        for reference in self.context_refs.iter().chain(&self.terminology_refs) {
            reference.validate()?;
        }
        if self.processing_location == ProcessingLocation::RemoteWithConsent
            && self.processing_consent_id.is_none()
        {
            return Err(ContractError::MissingProcessingConsent);
        }
        Ok(())
    }

    /// Checks that a request inherits a live session's policy and settings,
    /// and that every requested input span belongs to the exact input revision.
    pub fn validate_against_session(
        &self,
        session: &TranslationSession,
        source_assets: &[SourceAsset],
        source_spans: &[SourceSpan],
    ) -> Result<(), ContractError> {
        self.validate()?;
        session.validate()?;
        if self.session_id != session.record_id || self.session_revision != session.revision {
            return Err(ContractError::SessionReferenceMismatch);
        }
        if session.status != SessionStatus::Active {
            return Err(ContractError::SessionNotActive);
        }
        if self
            .target_languages
            .iter()
            .any(|language| !session.target_languages.contains(language))
            || session
                .context_refs
                .iter()
                .any(|reference| !self.context_refs.contains(reference))
            || session
                .terminology_refs
                .iter()
                .any(|reference| !self.terminology_refs.contains(reference))
            || self.processing_location != session.processing_location
        {
            return Err(ContractError::SessionPolicyMismatch);
        }
        if self
            .processing_consent_id
            .is_some_and(|grant_id| !session.consent_grant_ids.contains(&grant_id))
            || (self.processing_location == ProcessingLocation::RemoteWithConsent
                && self.processing_consent_id.is_none())
        {
            return Err(ContractError::MissingProcessingConsent);
        }

        let mut span_ids = std::collections::BTreeSet::new();
        let mut asset_ids = std::collections::BTreeSet::new();
        for span in source_spans {
            let asset = source_assets
                .iter()
                .find(|asset| asset.record_id == span.asset_id)
                .ok_or(ContractError::SourceAssetReferenceMismatch)?;
            span.validate_against(asset, session)?;
            if span.revision != self.input_revision
                || !self.source_span_ids.contains(&span.record_id)
                || !span_ids.insert(span.record_id)
            {
                return Err(ContractError::SourceSpanReferenceMismatch);
            }
            asset_ids.insert(span.asset_id);
        }
        if span_ids != self.source_span_ids.iter().copied().collect() {
            return Err(ContractError::SourceSpanReferenceMismatch);
        }
        let supplied_asset_ids: std::collections::BTreeSet<_> =
            source_assets.iter().map(|asset| asset.record_id).collect();
        if supplied_asset_ids.len() != source_assets.len() || supplied_asset_ids != asset_ids {
            return Err(ContractError::SourceAssetReferenceMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityState {
    Supported,
    Unsupported,
    Suspend,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityEvidence {
    pub capability_id: String,
    pub state: CapabilityState,
    pub engine_version: String,
    pub engine_revision: String,
    pub reason_code: String,
    pub observed_at: DateTime<Utc>,
}

impl CapabilityEvidence {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.capability_id.is_empty()
            || self.capability_id.chars().count() > 128
            || self.engine_version.is_empty()
            || self.engine_version.chars().count() > 128
            || self.reason_code.is_empty()
            || self.reason_code.chars().count() > 128
        {
            return Err(ContractError::InvalidEngineExecution);
        }
        validate_commit_sha(&self.engine_revision)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionOutcome {
    Completed,
    Unsupported,
    Suspend,
    Failed,
    Cancelled,
    TimedOut,
}

define_record!(EngineExecution {
    pub request_id: Uuid,
    pub engine_name: String,
    pub engine_version: String,
    pub engine_revision: String,
    pub public_entry_point: String,
    pub capability: CapabilityEvidence,
    pub input_fingerprint: String,
    pub response_fingerprint: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub outcome: ExecutionOutcome,
    pub diagnostic_code: Option<String>,
    pub trace_ref: Option<ContentReference>,
});

impl EngineExecution {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_base(&self.contract_version, &self.engine_name)?;
        if self.engine_name.is_empty()
            || self.engine_name.chars().count() > 128
            || self.engine_version.is_empty()
            || self.engine_version.chars().count() > 128
            || self.engine_revision.is_empty()
            || self.public_entry_point.is_empty()
            || self.public_entry_point.chars().count() > 256
            || self.capability.capability_id.is_empty()
            || self.capability.reason_code.is_empty()
            || self.input_fingerprint.is_empty()
            || self
                .diagnostic_code
                .as_ref()
                .is_some_and(|code| code.chars().count() > 128)
            || self.finished_at.is_none()
        {
            return Err(ContractError::InvalidEngineExecution);
        }
        self.capability.validate()?;
        validate_optional_sha256(Some(&self.input_fingerprint))?;
        validate_optional_sha256(self.response_fingerprint.as_deref())?;
        if self.capability.engine_version != self.engine_version
            || self.capability.engine_revision != self.engine_revision
        {
            return Err(ContractError::InvalidEngineExecution);
        }
        if let Some(finished_at) = self.finished_at
            && finished_at < self.started_at
        {
            return Err(ContractError::InvalidEngineExecution);
        }
        match (&self.capability.state, &self.outcome) {
            (CapabilityState::Supported, ExecutionOutcome::Completed)
                if self.response_fingerprint.is_some() =>
            {
                Ok(())
            }
            (CapabilityState::Supported, ExecutionOutcome::Failed)
            | (CapabilityState::Supported, ExecutionOutcome::Cancelled)
            | (CapabilityState::Supported, ExecutionOutcome::TimedOut) => Ok(()),
            (CapabilityState::Unsupported, ExecutionOutcome::Unsupported)
            | (CapabilityState::Suspend, ExecutionOutcome::Suspend) => Ok(()),
            _ => Err(ContractError::CapabilityOutcomeMismatch),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VariantStatus {
    Provisional,
    Final,
    Unsupported,
    Suspend,
    Failed,
    Cancelled,
    TimedOut,
}

define_record!(TranslationVariant {
    pub request_id: Uuid,
    pub source_span_id: Uuid,
    pub target_language: LanguageTag,
    pub revision: u64,
    pub status: VariantStatus,
    pub text: Option<String>,
    pub engine_execution_id: Option<Uuid>,
    pub provenance_ref: Option<ContentReference>,
    pub speaker_id: Option<Uuid>,
    pub position: Option<SourcePosition>,
    pub produced_at: DateTime<Utc>,
});

impl TranslationVariant {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_base(&self.contract_version, self.target_language.as_str())?;
        if self.revision == 0 {
            return Err(ContractError::InvalidVariant);
        }
        match self.status {
            VariantStatus::Final => {
                if self.text.as_deref().is_none_or(str::is_empty)
                    || self.engine_execution_id.is_none()
                {
                    return Err(ContractError::FinalVariantNeedsEvidence);
                }
            }
            VariantStatus::Provisional => {
                if self.text.as_deref().is_some_and(str::is_empty)
                    || (self.text.is_some() && self.engine_execution_id.is_none())
                {
                    return Err(ContractError::InvalidVariant);
                }
            }
            VariantStatus::Unsupported
            | VariantStatus::Suspend
            | VariantStatus::Failed
            | VariantStatus::Cancelled
            | VariantStatus::TimedOut => {
                if self.text.is_some() {
                    return Err(ContractError::TextOnUnsuccessfulVariant);
                }
            }
        }
        if let Some(position) = &self.position {
            position.validate()?;
        }
        if let Some(reference) = &self.provenance_ref {
            reference.validate()?;
        }
        Ok(())
    }

    pub fn validate_against(&self, execution: &EngineExecution) -> Result<(), ContractError> {
        self.validate()?;
        execution.validate()?;
        if self.engine_execution_id != Some(execution.record_id)
            || self.request_id != execution.request_id
        {
            return Err(ContractError::ExecutionReferenceMismatch);
        }
        let expected = match self.status {
            VariantStatus::Final | VariantStatus::Provisional => ExecutionOutcome::Completed,
            VariantStatus::Unsupported => ExecutionOutcome::Unsupported,
            VariantStatus::Suspend => ExecutionOutcome::Suspend,
            VariantStatus::Failed => ExecutionOutcome::Failed,
            VariantStatus::Cancelled => ExecutionOutcome::Cancelled,
            VariantStatus::TimedOut => ExecutionOutcome::TimedOut,
        };
        if execution.outcome != expected {
            return Err(ContractError::CapabilityOutcomeMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TranslationResultStatus {
    Pending,
    Processing,
    Completed,
    Partial,
    Unsupported,
    Suspend,
    Failed,
    Cancelled,
    TimedOut,
}

define_record!(TranslationResult {
    pub request_id: Uuid,
    pub request_revision: u64,
    pub status: TranslationResultStatus,
    pub variant_ids: Vec<Uuid>,
    pub engine_execution_ids: Vec<Uuid>,
    pub failure_code: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
});

impl TranslationResult {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_base(&self.contract_version, "TranslationResult")?;
        if self.request_revision == 0
            || self
                .failure_code
                .as_deref()
                .is_some_and(|code| code.is_empty() || code.chars().count() > 128)
        {
            return Err(ContractError::InvalidResult);
        }
        validate_unique(&self.variant_ids)?;
        validate_unique(&self.engine_execution_ids)?;

        match self.status {
            TranslationResultStatus::Pending | TranslationResultStatus::Processing => {
                if self.completed_at.is_some() {
                    return Err(ContractError::InvalidResult);
                }
            }
            TranslationResultStatus::Completed | TranslationResultStatus::Partial => {
                if self.completed_at.is_none()
                    || self.variant_ids.is_empty()
                    || self.engine_execution_ids.is_empty()
                {
                    return Err(ContractError::InvalidResult);
                }
            }
            TranslationResultStatus::Unsupported | TranslationResultStatus::Suspend => {
                if self.completed_at.is_none()
                    || self.failure_code.is_none()
                    || !self.variant_ids.is_empty()
                {
                    return Err(ContractError::InvalidResult);
                }
            }
            TranslationResultStatus::Failed
            | TranslationResultStatus::Cancelled
            | TranslationResultStatus::TimedOut => {
                if self.completed_at.is_none() || !self.variant_ids.is_empty() {
                    return Err(ContractError::InvalidResult);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationEventType {
    Accepted,
    Processing,
    VariantUpdated,
    Completed,
    Unsupported,
    Suspended,
    Failed,
    Cancelled,
    TimedOut,
    DeliveryUpdated,
}

define_record!(TranslationEvent {
    pub request_id: Uuid,
    pub sequence: u64,
    pub event_type: TranslationEventType,
    pub variant_id: Option<Uuid>,
    pub result_id: Option<Uuid>,
    pub delivery_artifact_id: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
});

impl TranslationEvent {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_base(&self.contract_version, "TranslationEvent")?;
        if self.sequence == 0
            || (self.event_type == TranslationEventType::VariantUpdated
                && self.variant_id.is_none())
            || (matches!(
                self.event_type,
                TranslationEventType::Completed
                    | TranslationEventType::Unsupported
                    | TranslationEventType::Suspended
                    | TranslationEventType::Failed
                    | TranslationEventType::Cancelled
                    | TranslationEventType::TimedOut
            ) && self.result_id.is_none())
            || (self.event_type == TranslationEventType::DeliveryUpdated
                && self.delivery_artifact_id.is_none())
        {
            return Err(ContractError::InvalidEvent);
        }
        Ok(())
    }
}

fn is_translation_terminal(event_type: &TranslationEventType) -> bool {
    matches!(
        event_type,
        TranslationEventType::Completed
            | TranslationEventType::Unsupported
            | TranslationEventType::Suspended
            | TranslationEventType::Failed
            | TranslationEventType::Cancelled
            | TranslationEventType::TimedOut
    )
}

/// Validates one page's local record ordering before merging it into a session
/// history. Lifecycle validation across pages requires `TranslationEventStream`.
/// Persistence must still allocate sequence numbers and append atomically.
pub fn validate_event_batch(events: &[TranslationEvent]) -> Result<(), ContractError> {
    let mut previous: Option<&TranslationEvent> = None;
    let mut terminal_seen = false;
    for event in events {
        event.validate()?;
        if let Some(previous) = previous
            && (event.request_id != previous.request_id
                || event.sequence <= previous.sequence
                || event.occurred_at < previous.occurred_at)
        {
            return Err(ContractError::UnorderedEvents);
        }
        let is_delivery = event.event_type == TranslationEventType::DeliveryUpdated;
        if terminal_seen && !is_delivery {
            return Err(ContractError::EventAfterTerminal);
        }
        if is_translation_terminal(&event.event_type) {
            terminal_seen = true;
        }
        previous = Some(event);
    }
    Ok(())
}

/// Tracks translation lifecycle across ordered pages for one request. This
/// cursor is process-local; durable checkpointing belongs to a storage adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranslationEventStream {
    request_id: Uuid,
    last_sequence: u64,
    last_occurred_at: Option<DateTime<Utc>>,
    terminal_seen: bool,
}

impl TranslationEventStream {
    pub fn new(request_id: Uuid) -> Self {
        Self {
            request_id,
            last_sequence: 0,
            last_occurred_at: None,
            terminal_seen: false,
        }
    }

    pub fn last_sequence(&self) -> u64 {
        self.last_sequence
    }

    pub fn terminal_seen(&self) -> bool {
        self.terminal_seen
    }

    /// Validates and applies a complete next page. The cursor changes only if
    /// every event passes, so a rejected page can be corrected and retried.
    pub fn apply_page(&mut self, events: &[TranslationEvent]) -> Result<(), ContractError> {
        let mut candidate = self.clone();
        for event in events {
            event.validate()?;
            let expected_sequence = candidate
                .last_sequence
                .checked_add(1)
                .ok_or(ContractError::UnorderedEvents)?;
            if event.request_id != candidate.request_id
                || event.sequence != expected_sequence
                || candidate
                    .last_occurred_at
                    .is_some_and(|previous| event.occurred_at < previous)
            {
                return Err(ContractError::UnorderedEvents);
            }

            if (candidate.last_sequence == 0 && event.event_type != TranslationEventType::Accepted)
                || (candidate.last_sequence > 0
                    && event.event_type == TranslationEventType::Accepted)
            {
                return Err(ContractError::AcceptedEventPosition);
            }
            if candidate.terminal_seen && event.event_type != TranslationEventType::DeliveryUpdated
            {
                return Err(ContractError::EventAfterTerminal);
            }
            if !candidate.terminal_seen && event.event_type == TranslationEventType::DeliveryUpdated
            {
                return Err(ContractError::EventBeforeTerminal);
            }

            if is_translation_terminal(&event.event_type) {
                candidate.terminal_seen = true;
            }
            candidate.last_sequence = event.sequence;
            candidate.last_occurred_at = Some(event.occurred_at);
        }
        *self = candidate;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactFormat {
    Text,
    Audio,
    Srt,
    Vtt,
    Document,
    Overlay,
    Api,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeliveryStatus {
    Prepared,
    Authorized,
    Delivered,
    DeliveryFailed,
    DeliveryUnknown,
    Revoked,
}

define_record!(DeliveryArtifact {
    pub variant_id: Uuid,
    pub format: ArtifactFormat,
    pub payload_ref: ContentReference,
    pub destination: ContentReference,
    pub position: Option<SourcePosition>,
    pub delivery_consent_id: Option<Uuid>,
    pub status: DeliveryStatus,
    pub delivered_at: Option<DateTime<Utc>>,
    pub audit_id: Uuid,
});

impl DeliveryArtifact {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_base(&self.contract_version, "DeliveryArtifact")?;
        if self.status == DeliveryStatus::Delivered {
            if self.delivery_consent_id.is_none() || self.delivered_at.is_none() {
                return Err(ContractError::DeliveryNeedsAuthorization);
            }
        } else if self.delivered_at.is_some() {
            return Err(ContractError::UnexpectedDeliveryTime);
        }
        if matches!(
            self.status,
            DeliveryStatus::Authorized
                | DeliveryStatus::Delivered
                | DeliveryStatus::DeliveryUnknown
                | DeliveryStatus::Revoked
        ) && self.delivery_consent_id.is_none()
        {
            return Err(ContractError::DeliveryNeedsAuthorization);
        }
        self.payload_ref.validate()?;
        self.destination.validate()?;
        if let Some(position) = &self.position {
            position.validate()?;
        }
        Ok(())
    }
}

/// Closed record sum type used by language-neutral transports.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "record_type")]
pub enum LogicalRecord {
    #[serde(rename = "SourceAsset")]
    SourceAsset(SourceAsset),
    #[serde(rename = "ConsentGrant")]
    ConsentGrant(ConsentGrant),
    #[serde(rename = "ConsentRevocation")]
    ConsentRevocation(ConsentRevocation),
    #[serde(rename = "TranslationSession")]
    TranslationSession(TranslationSession),
    #[serde(rename = "SourceSpan")]
    SourceSpan(SourceSpan),
    #[serde(rename = "TranslationRequest")]
    TranslationRequest(TranslationRequest),
    #[serde(rename = "TranslationResult")]
    TranslationResult(TranslationResult),
    #[serde(rename = "TranslationEvent")]
    TranslationEvent(TranslationEvent),
    #[serde(rename = "EngineExecution")]
    EngineExecution(EngineExecution),
    #[serde(rename = "TranslationVariant")]
    TranslationVariant(TranslationVariant),
    #[serde(rename = "DeliveryArtifact")]
    DeliveryArtifact(DeliveryArtifact),
}

impl LogicalRecord {
    pub fn validate(&self) -> Result<(), ContractError> {
        match self {
            Self::SourceAsset(record) => record.validate(),
            Self::ConsentGrant(record) => record.validate(),
            Self::ConsentRevocation(record) => record.validate(),
            Self::TranslationSession(record) => record.validate(),
            Self::SourceSpan(record) => record.validate(),
            Self::TranslationRequest(record) => record.validate(),
            Self::TranslationResult(record) => record.validate(),
            Self::TranslationEvent(record) => record.validate(),
            Self::EngineExecution(record) => record.validate(),
            Self::TranslationVariant(record) => record.validate(),
            Self::DeliveryArtifact(record) => record.validate(),
        }
    }

    /// Converts a validated logical record to the public exchange envelope.
    /// The logical contract version is not serialized as the wire schema version.
    pub fn to_exchange_value(&self) -> Result<serde_json::Value, ExchangeCodecError> {
        self.validate()?;
        let mut value = serde_json::to_value(self)?;
        let object = value
            .as_object_mut()
            .ok_or(ExchangeCodecError::InvalidEnvelope)?;
        if object.remove("contract_version").is_none() {
            return Err(ExchangeCodecError::InvalidEnvelope);
        }
        object.insert(
            "schema_version".into(),
            serde_json::Value::String(EXCHANGE_SCHEMA_VERSION.into()),
        );
        Ok(value)
    }

    /// Converts a public exchange envelope into a validated logical record.
    pub fn from_exchange_value(mut value: serde_json::Value) -> Result<Self, ExchangeCodecError> {
        let object = value
            .as_object_mut()
            .ok_or(ExchangeCodecError::InvalidEnvelope)?;
        let schema_version = object
            .remove("schema_version")
            .and_then(|version| version.as_str().map(str::to_owned))
            .ok_or(ExchangeCodecError::InvalidEnvelope)?;
        if schema_version != EXCHANGE_SCHEMA_VERSION {
            return Err(ExchangeCodecError::UnsupportedSchemaVersion(schema_version));
        }
        if object.contains_key("contract_version") {
            return Err(ExchangeCodecError::InvalidEnvelope);
        }
        object.insert(
            "contract_version".into(),
            serde_json::Value::String(LOGICAL_CONTRACT_VERSION.into()),
        );
        let record: Self = serde_json::from_value(value)?;
        record.validate()?;
        Ok(record)
    }
}

#[derive(Debug, Error)]
pub enum ExchangeCodecError {
    #[error("exchange record envelope is malformed")]
    InvalidEnvelope,
    #[error("unsupported exchange schema version: {0}")]
    UnsupportedSchemaVersion(String),
    #[error("logical record validation failed: {0}")]
    Contract(#[from] ContractError),
    #[error("record JSON conversion failed: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ContractError {
    #[error("unsupported logical/schema contract version")]
    UnsupportedSchemaVersion,
    #[error("required field is empty")]
    EmptyRequiredField,
    #[error("language identifier is not a valid BCP 47 tag")]
    InvalidLanguageTag,
    #[error("duplicate values violate request identity")]
    DuplicateValue,
    #[error("capture permission evidence is missing")]
    MissingCaptureConsent,
    #[error("remote processing requires an explicit consent reference")]
    MissingProcessingConsent,
    #[error("consent grant fields are incomplete or inconsistent")]
    InvalidConsent,
    #[error("consent revocation fields are incomplete or inconsistent")]
    InvalidRevocation,
    #[error("translation result fields or terminal state are incomplete or inconsistent")]
    InvalidResult,
    #[error("translation event fields are incomplete or inconsistent")]
    InvalidEvent,
    #[error("translation events are duplicated, unordered, or cross request boundaries")]
    UnorderedEvents,
    #[error("translation event follows a terminal event")]
    EventAfterTerminal,
    #[error("delivery event precedes translation termination")]
    EventBeforeTerminal,
    #[error("accepted event must be the first and only accepted event")]
    AcceptedEventPosition,
    #[error("session fields are incomplete or inconsistent")]
    InvalidSession,
    #[error("source span fields are incomplete or inconsistent")]
    InvalidSourceSpan,
    #[error("source asset fields or metadata values are invalid")]
    InvalidSourceAsset,
    #[error("content reference fields are incomplete or exceed their limits")]
    InvalidReference,
    #[error("source confidence must be finite and between zero and one")]
    InvalidConfidence,
    #[error("source media position is invalid")]
    InvalidSourcePosition,
    #[error("translation request fields are incomplete or inconsistent")]
    InvalidRequest,
    #[error("request does not reference the exact session revision")]
    SessionReferenceMismatch,
    #[error("new request cannot run in a paused or closed session")]
    SessionNotActive,
    #[error("request conflicts with inherited session settings or processing policy")]
    SessionPolicyMismatch,
    #[error("request source spans do not match the exact input revision")]
    SourceSpanReferenceMismatch,
    #[error("source span does not resolve to its source asset")]
    SourceAssetReferenceMismatch,
    #[error("engine execution evidence is incomplete or inconsistent")]
    InvalidEngineExecution,
    #[error("engine capability and execution outcome disagree")]
    CapabilityOutcomeMismatch,
    #[error("translation variant fields are incomplete or inconsistent")]
    InvalidVariant,
    #[error("a final translation requires nonempty text and engine evidence")]
    FinalVariantNeedsEvidence,
    #[error("an unsuccessful variant cannot carry translation text")]
    TextOnUnsuccessfulVariant,
    #[error("variant and engine execution references disagree")]
    ExecutionReferenceMismatch,
    #[error("delivery completion requires a distinct authorization and timestamp")]
    DeliveryNeedsAuthorization,
    #[error("non-delivered artifact cannot claim a delivery timestamp")]
    UnexpectedDeliveryTime,
    #[error("SHA-256 fingerprint must contain exactly 64 lowercase hexadecimal digits")]
    InvalidSha256,
}

fn validate_base(contract_version: &str, required: &str) -> Result<(), ContractError> {
    if contract_version != LOGICAL_CONTRACT_VERSION {
        return Err(ContractError::UnsupportedSchemaVersion);
    }
    if required.is_empty() {
        return Err(ContractError::EmptyRequiredField);
    }
    Ok(())
}

fn validate_unique<T: Ord>(values: &[T]) -> Result<(), ContractError> {
    let unique: std::collections::BTreeSet<_> = values.iter().collect();
    if unique.len() != values.len() {
        return Err(ContractError::DuplicateValue);
    }
    Ok(())
}

fn validate_optional_sha256(value: Option<&str>) -> Result<(), ContractError> {
    if value.is_some_and(|digest| {
        digest.len() != 64
            || !digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }) {
        return Err(ContractError::InvalidSha256);
    }
    Ok(())
}

fn validate_commit_sha(value: &str) -> Result<(), ContractError> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ContractError::InvalidEngineExecution);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 9, hour, 0, 0).unwrap()
    }

    fn request(location: ProcessingLocation, consent: Option<Uuid>) -> TranslationRequest {
        TranslationRequest {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(1),
            created_at: at(1),
            session_id: Uuid::from_u128(2),
            session_revision: 1,
            revision: 1,
            source_span_ids: vec![Uuid::from_u128(3)],
            input_revision: 1,
            target_languages: vec!["en".parse().unwrap()],
            context_refs: vec![],
            terminology_refs: vec![],
            requested_outputs: vec![RequestedOutput::Text],
            execution_mode: ExecutionMode::Batch,
            processing_location: location,
            processing_consent_id: consent,
            idempotency_key: "request-identity-0001".into(),
            attempt: 1,
            deadline: None,
            cancel_requested_at: None,
        }
    }

    fn unsupported_execution() -> EngineExecution {
        EngineExecution {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(4),
            created_at: at(1),
            request_id: Uuid::from_u128(5),
            engine_name: "MINIDORA-BETA".into(),
            engine_version: "0.6.0b1".into(),
            engine_revision: "fe5822de63644bdaea9208a98046f71684dffae2".into(),
            public_entry_point: "minidora.engine_beta.ミニドラβ実行機.実行".into(),
            capability: CapabilityEvidence {
                capability_id: "translation.text".into(),
                state: CapabilityState::Unsupported,
                engine_version: "0.6.0b1".into(),
                engine_revision: "fe5822de63644bdaea9208a98046f71684dffae2".into(),
                reason_code: "NO_PUBLIC_TRANSLATION_OPERATION".into(),
                observed_at: at(1),
            },
            input_fingerprint: "0000000000000000000000000000000000000000000000000000000000000000"
                .into(),
            response_fingerprint: None,
            started_at: at(1),
            finished_at: Some(at(1)),
            outcome: ExecutionOutcome::Unsupported,
            diagnostic_code: Some("NO_PUBLIC_TRANSLATION_OPERATION".into()),
            trace_ref: None,
        }
    }

    fn source_asset() -> SourceAsset {
        SourceAsset {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(40),
            created_at: at(1),
            media_kind: MediaKind::Image,
            origin: InputOrigin::Camera,
            media_type: "image/png".into(),
            content_ref: Some(ContentReference {
                kind: ReferenceKind::Local,
                key: "asset/40".into(),
                media_type: Some("image/png".into()),
                sha256: None,
                size_bytes: Some(128),
            }),
            original_fingerprint: Some("a".repeat(64)),
            owner_ref: Some(ContentReference {
                kind: ReferenceKind::Local,
                key: "owner/40".into(),
                media_type: None,
                sha256: None,
                size_bytes: None,
            }),
            rights_ref: None,
            capture_consent_id: Some(Uuid::from_u128(41)),
            time_basis: Some("unix_ms_from_capture_start".into()),
            retention: RetentionPolicy::Session,
            metadata: BTreeMap::from([("device".into(), serde_json::json!("camera"))]),
        }
    }

    #[test]
    fn source_asset_limits_and_scalar_metadata_are_validated() {
        let mut asset = source_asset();
        assert_eq!(asset.validate(), Ok(()));

        asset.capture_consent_id = None;
        assert_eq!(asset.validate(), Err(ContractError::MissingCaptureConsent));

        asset = source_asset();
        asset.metadata.insert(
            "nested".into(),
            serde_json::json!({"raw_payload": "must not be metadata"}),
        );
        assert_eq!(asset.validate(), Err(ContractError::InvalidSourceAsset));

        asset = source_asset();
        asset.media_type = "x".repeat(128);
        assert_eq!(asset.validate(), Err(ContractError::InvalidSourceAsset));

        asset = source_asset();
        asset.original_fingerprint = Some("A".repeat(64));
        assert_eq!(asset.validate(), Err(ContractError::InvalidSha256));

        asset = source_asset();
        asset.content_ref.as_mut().unwrap().key = "x".repeat(1025);
        assert_eq!(asset.validate(), Err(ContractError::InvalidReference));
    }

    #[test]
    fn source_span_uses_tagged_payload_and_position_records() {
        let span = SourceSpan {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(50),
            created_at: at(1),
            asset_id: Uuid::from_u128(51),
            session_id: Uuid::from_u128(52),
            revision: 1,
            payload: SpanPayload::Text {
                text: "こんにちは".into(),
            },
            source_language: Some("ja".parse().unwrap()),
            observation_origin: ObservationOrigin::Typed,
            confidence: None,
            speaker_id: None,
            position: Some(SourcePosition::AudioTime {
                start_ms: 0,
                end_ms: 100,
            }),
            provenance_ref: None,
        };
        assert_eq!(span.validate(), Ok(()));

        let encoded = serde_json::to_value(&span).unwrap();
        assert_eq!(encoded["payload"]["kind"], "text");
        assert_eq!(encoded["position"]["kind"], "audio_time");
        assert_eq!(serde_json::from_value::<SourceSpan>(encoded).unwrap(), span);

        let mut invalid = span.clone();
        invalid.payload = SpanPayload::Text {
            text: String::new(),
        };
        assert_eq!(invalid.validate(), Err(ContractError::InvalidSourceSpan));
        invalid.payload = span.payload;
        invalid.position = Some(SourcePosition::AudioTime {
            start_ms: 100,
            end_ms: 0,
        });
        assert_eq!(
            invalid.validate(),
            Err(ContractError::InvalidSourcePosition)
        );

        invalid.position = Some(SourcePosition::ImageBounds {
            page: None,
            x: 0.8,
            y: 0.1,
            width: 0.3,
            height: 0.2,
        });
        assert_eq!(
            invalid.validate(),
            Err(ContractError::InvalidSourcePosition)
        );
        assert!(serde_json::from_str::<ObservationOrigin>("\"typed_fixture\"").is_err());
    }

    #[test]
    fn engine_execution_preserves_nested_capability_evidence_on_the_wire() {
        let execution = unsupported_execution();
        assert_eq!(execution.validate(), Ok(()));

        let wire = serde_json::to_value(&execution).unwrap();
        assert_eq!(wire["capability"]["state"], "UNSUPPORTED");
        assert_eq!(wire["input_fingerprint"], execution.input_fingerprint);
        assert_eq!(
            serde_json::from_value::<EngineExecution>(wire).unwrap(),
            execution
        );

        let mut invalid = execution.clone();
        invalid.engine_name = "x".repeat(129);
        assert_eq!(
            invalid.validate(),
            Err(ContractError::InvalidEngineExecution)
        );
        invalid = execution;
        invalid.capability.reason_code = "x".repeat(129);
        assert_eq!(
            invalid.validate(),
            Err(ContractError::InvalidEngineExecution)
        );
    }

    #[test]
    fn exchange_codec_keeps_wire_and_logical_versions_distinct() {
        let record =
            LogicalRecord::TranslationRequest(request(ProcessingLocation::LocalOnly, None));
        let encoded = record.to_exchange_value().unwrap();
        assert_eq!(encoded["schema_version"], EXCHANGE_SCHEMA_VERSION);
        assert!(encoded.get("contract_version").is_none());
        assert_eq!(encoded["record_type"], "TranslationRequest");
        assert_eq!(
            LogicalRecord::from_exchange_value(encoded.clone()).unwrap(),
            record
        );

        let mut unsupported = encoded.clone();
        unsupported["schema_version"] = serde_json::Value::String("2.0.0".into());
        assert!(matches!(
            LogicalRecord::from_exchange_value(unsupported),
            Err(ExchangeCodecError::UnsupportedSchemaVersion(version)) if version == "2.0.0"
        ));

        let mut leaked_contract_version = encoded;
        leaked_contract_version["contract_version"] =
            serde_json::Value::String(LOGICAL_CONTRACT_VERSION.into());
        assert!(matches!(
            LogicalRecord::from_exchange_value(leaked_contract_version),
            Err(ExchangeCodecError::InvalidEnvelope)
        ));

        let mut missing_version = record.to_exchange_value().unwrap();
        missing_version
            .as_object_mut()
            .unwrap()
            .remove("schema_version");
        assert!(matches!(
            LogicalRecord::from_exchange_value(missing_version),
            Err(ExchangeCodecError::InvalidEnvelope)
        ));

        let mut unknown_field = record.to_exchange_value().unwrap();
        unknown_field["unexpected"] = serde_json::Value::Bool(true);
        assert!(matches!(
            LogicalRecord::from_exchange_value(unknown_field),
            Err(ExchangeCodecError::Serde(_))
        ));
    }

    #[test]
    fn language_tags_are_validated_and_canonicalized() {
        assert_eq!(
            "zh-hant-tw".parse::<LanguageTag>().unwrap().as_str(),
            "zh-Hant-TW"
        );
        for valid in [
            "zh-cmn-Hans-CN",
            "de-CH-1901",
            "sl-rozaj-biske",
            "en-u-ca-gregory",
            "x-private",
            "i-KLINGON",
            "en-gb-OED",
        ] {
            assert!(valid.parse::<LanguageTag>().is_ok(), "{valid}");
        }
        assert_eq!(
            "i-KLINGON".parse::<LanguageTag>().unwrap().as_str(),
            "i-klingon"
        );
        for invalid in [
            "not a tag",
            "en--US",
            "en-a",
            "en-x",
            "en-US-Latn",
            "en-u-ca-u-nu",
            "sl-rozaj-rozaj",
            "x",
            "a-bbb",
        ] {
            assert!(invalid.parse::<LanguageTag>().is_err(), "{invalid}");
        }
        assert!(serde_json::from_str::<LanguageTag>("\"en--US\"").is_err());
    }

    #[test]
    fn unsupported_result_has_a_terminal_reason_and_no_variants() {
        let result = TranslationResult {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(9),
            created_at: at(1),
            request_id: Uuid::from_u128(1),
            request_revision: 1,
            status: TranslationResultStatus::Unsupported,
            variant_ids: vec![],
            engine_execution_ids: vec![],
            failure_code: Some("NO_PUBLIC_TRANSLATION_OPERATION".into()),
            completed_at: Some(at(2)),
        };
        assert_eq!(result.validate(), Ok(()));

        let mut fabricated = result;
        fabricated.variant_ids.push(Uuid::from_u128(10));
        assert_eq!(fabricated.validate(), Err(ContractError::InvalidResult));
    }

    #[test]
    fn translation_result_status_controls_completion_and_failure_evidence() {
        let mut result = TranslationResult {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(80),
            created_at: at(1),
            request_id: Uuid::from_u128(1),
            request_revision: 1,
            status: TranslationResultStatus::Pending,
            variant_ids: vec![],
            engine_execution_ids: vec![],
            failure_code: None,
            completed_at: None,
        };
        assert_eq!(result.validate(), Ok(()));
        let wire = serde_json::to_value(&result).unwrap();
        assert!(wire["failure_code"].is_null());
        assert!(wire["completed_at"].is_null());

        result.status = TranslationResultStatus::Processing;
        assert_eq!(result.validate(), Ok(()));
        result.completed_at = Some(at(2));
        assert_eq!(result.validate(), Err(ContractError::InvalidResult));
        result.completed_at = None;

        result.status = TranslationResultStatus::Completed;
        assert_eq!(result.validate(), Err(ContractError::InvalidResult));
        result.completed_at = Some(at(2));
        result.variant_ids.push(Uuid::from_u128(81));
        result.engine_execution_ids.push(Uuid::from_u128(82));
        assert_eq!(result.validate(), Ok(()));

        result.failure_code = Some(String::new());
        assert_eq!(result.validate(), Err(ContractError::InvalidResult));
        result.failure_code = Some("x".repeat(129));
        assert_eq!(result.validate(), Err(ContractError::InvalidResult));
        result.failure_code = None;

        result.status = TranslationResultStatus::Unsupported;
        result.variant_ids.clear();
        result.engine_execution_ids.clear();
        assert_eq!(result.validate(), Err(ContractError::InvalidResult));
        result.failure_code = Some("CAPABILITY_NOT_EXPOSED".into());
        assert_eq!(result.validate(), Ok(()));
    }

    #[test]
    fn events_keep_order_and_allow_delivery_after_translation_terminal() {
        let make_event = |sequence, event_type, result_id, delivery_artifact_id| TranslationEvent {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(100 + sequence as u128),
            created_at: at(1),
            request_id: Uuid::from_u128(1),
            sequence,
            event_type,
            variant_id: None,
            result_id,
            delivery_artifact_id,
            occurred_at: at(sequence as u32),
        };
        let events = vec![
            make_event(1, TranslationEventType::Accepted, None, None),
            make_event(
                2,
                TranslationEventType::Completed,
                Some(Uuid::from_u128(9)),
                None,
            ),
            make_event(
                3,
                TranslationEventType::DeliveryUpdated,
                None,
                Some(Uuid::from_u128(11)),
            ),
        ];
        assert_eq!(validate_event_batch(&events), Ok(()));

        let mut invalid = events;
        invalid.push(make_event(4, TranslationEventType::Processing, None, None));
        assert_eq!(
            validate_event_batch(&invalid),
            Err(ContractError::EventAfterTerminal)
        );
    }

    #[test]
    fn event_stream_preserves_terminal_state_across_pages() {
        let request_id = Uuid::from_u128(70);
        let make_event = |sequence, event_type, result_id, delivery_artifact_id| TranslationEvent {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(1_000 + sequence as u128),
            created_at: at(1),
            request_id,
            sequence,
            event_type,
            variant_id: None,
            result_id,
            delivery_artifact_id,
            occurred_at: at(sequence as u32),
        };
        let mut stream = TranslationEventStream::new(request_id);
        assert_eq!(
            stream.apply_page(&[
                make_event(1, TranslationEventType::Accepted, None, None),
                make_event(2, TranslationEventType::Processing, None, None),
            ]),
            Ok(())
        );

        assert_eq!(
            stream.apply_page(&[make_event(
                3,
                TranslationEventType::DeliveryUpdated,
                None,
                Some(Uuid::from_u128(80)),
            )]),
            Err(ContractError::EventBeforeTerminal)
        );
        assert_eq!(stream.last_sequence(), 2);

        assert_eq!(
            stream.apply_page(&[make_event(
                3,
                TranslationEventType::Completed,
                Some(Uuid::from_u128(81)),
                None,
            )]),
            Ok(())
        );
        assert!(stream.terminal_seen());
        assert_eq!(
            stream.apply_page(&[make_event(
                4,
                TranslationEventType::DeliveryUpdated,
                None,
                Some(Uuid::from_u128(80)),
            )]),
            Ok(())
        );
        assert_eq!(
            stream.apply_page(&[make_event(
                5,
                TranslationEventType::Failed,
                Some(Uuid::from_u128(82)),
                None,
            )]),
            Err(ContractError::EventAfterTerminal)
        );
        assert_eq!(stream.last_sequence(), 4);
    }

    #[test]
    fn event_stream_rejects_gaps_and_duplicate_acceptance_atomically() {
        let request_id = Uuid::from_u128(90);
        let make_event = |sequence, event_type| TranslationEvent {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(2_000 + sequence as u128),
            created_at: at(1),
            request_id,
            sequence,
            event_type,
            variant_id: None,
            result_id: None,
            delivery_artifact_id: None,
            occurred_at: at(sequence as u32),
        };
        let mut stream = TranslationEventStream::new(request_id);
        assert_eq!(
            stream.apply_page(&[make_event(1, TranslationEventType::Accepted)]),
            Ok(())
        );
        assert_eq!(
            stream.apply_page(&[
                make_event(2, TranslationEventType::Processing),
                make_event(4, TranslationEventType::Processing),
            ]),
            Err(ContractError::UnorderedEvents)
        );
        assert_eq!(stream.last_sequence(), 1);
        assert_eq!(
            stream.apply_page(&[make_event(2, TranslationEventType::Accepted)]),
            Err(ContractError::AcceptedEventPosition)
        );
        assert_eq!(stream.last_sequence(), 1);
    }

    #[test]
    fn remote_processing_cannot_be_constructed_without_consent_reference() {
        assert_eq!(
            request(ProcessingLocation::RemoteWithConsent, None).validate(),
            Err(ContractError::MissingProcessingConsent)
        );
    }

    #[test]
    fn consent_principal_and_revocation_fields_match_wire_limits() {
        let mut grant = ConsentGrant {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(91),
            created_at: at(1),
            principal_id: "owner".into(),
            scope: ConsentScope::ProcessLocal,
            subject_ids: vec![Uuid::from_u128(92)],
            granted_at: at(1),
            expires_at: None,
            evidence_ref: None,
        };
        grant.principal_id = "語".repeat(256);
        assert_eq!(grant.validate(), Ok(()));
        grant.principal_id.push('語');
        assert_eq!(grant.validate(), Err(ContractError::InvalidConsent));

        let mut revocation = ConsentRevocation {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(93),
            created_at: at(1),
            grant_id: Uuid::from_u128(92),
            revoked_by: "語".repeat(256),
            revoked_at: at(2),
            reason_code: "語".repeat(128),
        };
        assert_eq!(revocation.validate(), Ok(()));
        revocation.revoked_by.push('語');
        assert_eq!(revocation.validate(), Err(ContractError::InvalidRevocation));
        revocation.revoked_by = "owner".into();
        revocation.reason_code.push('語');
        assert_eq!(revocation.validate(), Err(ContractError::InvalidRevocation));
    }

    #[test]
    fn request_deadline_and_cancellation_cannot_precede_a_valid_request_window() {
        let mut invalid_deadline = request(ProcessingLocation::LocalOnly, None);
        invalid_deadline.deadline = Some(at(1));
        assert_eq!(
            invalid_deadline.validate(),
            Err(ContractError::InvalidRequest)
        );

        let mut invalid_cancellation = request(ProcessingLocation::LocalOnly, None);
        invalid_cancellation.cancel_requested_at = Some(at(0));
        assert_eq!(
            invalid_cancellation.validate(),
            Err(ContractError::InvalidRequest)
        );

        let mut boundary_request = request(ProcessingLocation::LocalOnly, None);
        boundary_request.session_revision = u64::MAX;
        boundary_request.revision = u64::MAX;
        boundary_request.input_revision = u64::MAX;
        boundary_request.attempt = u32::MAX;
        boundary_request.idempotency_key = "x".repeat(256);
        boundary_request.deadline = Some(at(2));
        boundary_request.cancel_requested_at = Some(at(1));
        assert_eq!(boundary_request.validate(), Ok(()));

        boundary_request.idempotency_key = "x".repeat(257);
        assert_eq!(
            boundary_request.validate(),
            Err(ContractError::InvalidRequest)
        );
        boundary_request.idempotency_key = "界".repeat(15);
        assert_eq!(
            boundary_request.validate(),
            Err(ContractError::InvalidRequest)
        );
        boundary_request.idempotency_key = "界".repeat(16);
        assert_eq!(boundary_request.validate(), Ok(()));
    }

    #[test]
    fn unsupported_execution_cannot_be_reported_as_completed() {
        let mut evidence = unsupported_execution();
        evidence.outcome = ExecutionOutcome::Completed;
        evidence.response_fingerprint =
            Some("1111111111111111111111111111111111111111111111111111111111111111".into());
        assert_eq!(
            evidence.validate(),
            Err(ContractError::CapabilityOutcomeMismatch)
        );
    }

    #[test]
    fn unsupported_translation_cannot_carry_fabricated_text() {
        let variant = TranslationVariant {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(6),
            created_at: at(1),
            request_id: Uuid::from_u128(5),
            source_span_id: Uuid::from_u128(3),
            target_language: "en".parse().unwrap(),
            revision: 1,
            status: VariantStatus::Unsupported,
            text: Some("fabricated".into()),
            engine_execution_id: Some(Uuid::from_u128(4)),
            provenance_ref: None,
            speaker_id: None,
            position: None,
            produced_at: at(1),
        };
        assert_eq!(
            variant.validate(),
            Err(ContractError::TextOnUnsuccessfulVariant)
        );
    }

    #[test]
    fn provisional_translation_can_precede_text_and_engine_evidence() {
        let mut variant = TranslationVariant {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(60),
            created_at: at(1),
            request_id: Uuid::from_u128(5),
            source_span_id: Uuid::from_u128(3),
            target_language: "en".parse().unwrap(),
            revision: 1,
            status: VariantStatus::Provisional,
            text: None,
            engine_execution_id: None,
            provenance_ref: None,
            speaker_id: None,
            position: None,
            produced_at: at(1),
        };
        assert_eq!(variant.validate(), Ok(()));
        let wire = serde_json::to_value(&variant).unwrap();
        assert!(wire["text"].is_null());
        assert!(wire["engine_execution_id"].is_null());

        variant.text = Some("interim text".into());
        assert_eq!(variant.validate(), Err(ContractError::InvalidVariant));
        variant.engine_execution_id = Some(Uuid::from_u128(61));
        assert_eq!(variant.validate(), Ok(()));

        variant.status = VariantStatus::Final;
        variant.engine_execution_id = None;
        assert_eq!(
            variant.validate(),
            Err(ContractError::FinalVariantNeedsEvidence)
        );
    }

    #[test]
    fn final_translation_must_point_to_the_successful_engine_execution() {
        let variant = TranslationVariant {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(6),
            created_at: at(1),
            request_id: Uuid::from_u128(5),
            source_span_id: Uuid::from_u128(3),
            target_language: "en".parse().unwrap(),
            revision: 1,
            status: VariantStatus::Final,
            text: Some("result".into()),
            engine_execution_id: Some(Uuid::from_u128(4)),
            provenance_ref: None,
            speaker_id: None,
            position: None,
            produced_at: at(1),
        };
        assert_eq!(
            variant.validate_against(&unsupported_execution()),
            Err(ContractError::CapabilityOutcomeMismatch)
        );
    }

    #[test]
    fn delivered_artifact_requires_a_separate_delivery_grant() {
        let artifact = DeliveryArtifact {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(7),
            created_at: at(1),
            variant_id: Uuid::from_u128(6),
            format: ArtifactFormat::Text,
            payload_ref: ContentReference {
                kind: ReferenceKind::Local,
                key: "artifact/result-1".into(),
                media_type: Some("text/plain".into()),
                sha256: None,
                size_bytes: None,
            },
            destination: ContentReference {
                kind: ReferenceKind::Remote,
                key: "recipient:example".into(),
                media_type: None,
                sha256: None,
                size_bytes: None,
            },
            position: None,
            delivery_consent_id: None,
            status: DeliveryStatus::Delivered,
            delivered_at: Some(at(2)),
            audit_id: Uuid::from_u128(8),
        };
        assert_eq!(
            artifact.validate(),
            Err(ContractError::DeliveryNeedsAuthorization)
        );
    }

    #[test]
    fn delivery_artifact_validates_payload_position_and_logical_statuses() {
        let mut artifact = DeliveryArtifact {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(70),
            created_at: at(1),
            variant_id: Uuid::from_u128(71),
            format: ArtifactFormat::Vtt,
            payload_ref: ContentReference {
                kind: ReferenceKind::Local,
                key: "artifact/captions.vtt".into(),
                media_type: Some("text/vtt".into()),
                sha256: None,
                size_bytes: Some(10),
            },
            destination: ContentReference {
                kind: ReferenceKind::Remote,
                key: "recipient:meeting-1".into(),
                media_type: None,
                sha256: None,
                size_bytes: None,
            },
            position: Some(SourcePosition::VideoTime {
                start_ms: 100,
                end_ms: 500,
            }),
            delivery_consent_id: None,
            status: DeliveryStatus::Prepared,
            delivered_at: None,
            audit_id: Uuid::from_u128(72),
        };
        assert_eq!(artifact.validate(), Ok(()));

        let wire = serde_json::to_value(LogicalRecord::DeliveryArtifact(artifact.clone())).unwrap();
        assert_eq!(wire["status"], "PREPARED");
        assert_eq!(wire["destination"]["kind"], "remote");
        assert_eq!(wire["payload_ref"]["key"], "artifact/captions.vtt");
        assert_eq!(wire["position"]["kind"], "video_time");
        assert!(wire["delivery_consent_id"].is_null());
        assert!(wire["delivered_at"].is_null());
        assert_eq!(wire["audit_id"], Uuid::from_u128(72).to_string());

        let valid_payload_ref = artifact.payload_ref.clone();
        artifact.payload_ref.key.clear();
        assert_eq!(artifact.validate(), Err(ContractError::InvalidReference));
        artifact.payload_ref = valid_payload_ref;

        let valid_destination = artifact.destination.clone();
        artifact.destination.key.clear();
        assert_eq!(artifact.validate(), Err(ContractError::InvalidReference));
        artifact.destination = valid_destination;

        artifact.status = DeliveryStatus::Authorized;
        assert_eq!(
            artifact.validate(),
            Err(ContractError::DeliveryNeedsAuthorization)
        );
        artifact.delivery_consent_id = Some(Uuid::from_u128(73));
        assert_eq!(artifact.validate(), Ok(()));

        artifact.status = DeliveryStatus::Delivered;
        assert_eq!(
            artifact.validate(),
            Err(ContractError::DeliveryNeedsAuthorization)
        );
        artifact.delivered_at = Some(at(2));
        assert_eq!(artifact.validate(), Ok(()));

        artifact.status = DeliveryStatus::DeliveryUnknown;
        artifact.delivered_at = None;
        assert_eq!(artifact.validate(), Ok(()));
        artifact.position = Some(SourcePosition::VideoTime {
            start_ms: 500,
            end_ms: 100,
        });
        assert_eq!(
            artifact.validate(),
            Err(ContractError::InvalidSourcePosition)
        );
    }
}
