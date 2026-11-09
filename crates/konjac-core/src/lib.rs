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
pub mod engine;

pub const LOGICAL_CONTRACT_VERSION: &str = "1.0.0";

/// A validated BCP 47 language identifier, kept separate from display names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct LanguageTag(String);

impl LanguageTag {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for LanguageTag {
    type Err = ContractError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split('-');
        let language = parts.next().ok_or(ContractError::InvalidLanguageTag)?;
        if !(2..=8).contains(&language.len()) || !language.bytes().all(|b| b.is_ascii_alphabetic())
        {
            return Err(ContractError::InvalidLanguageTag);
        }

        let mut canonical = vec![language.to_ascii_lowercase()];
        for part in parts {
            if part.is_empty() || part.len() > 8 || !part.bytes().all(|b| b.is_ascii_alphanumeric())
            {
                return Err(ContractError::InvalidLanguageTag);
            }
            let value = if part.len() == 4 && part.bytes().all(|b| b.is_ascii_alphabetic()) {
                let mut chars = part.chars();
                let first = chars
                    .next()
                    .ok_or(ContractError::InvalidLanguageTag)?
                    .to_ascii_uppercase();
                format!("{first}{}", chars.as_str().to_ascii_lowercase())
            } else if (part.len() == 2 && part.bytes().all(|b| b.is_ascii_alphabetic()))
                || (part.len() == 3 && part.bytes().all(|b| b.is_ascii_digit()))
            {
                part.to_ascii_uppercase()
            } else {
                part.to_ascii_lowercase()
            };
            canonical.push(value);
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
        if self.key.is_empty() || self.media_type.as_deref().is_some_and(str::is_empty) {
            return Err(ContractError::EmptyRequiredField);
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
        if self.subject_ids.is_empty()
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
        if self.reason_code.is_empty() {
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
            Self::DocumentAnchor { anchor, page } if anchor.is_empty() || page == &Some(0) => {
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
    pub observation_origin: String,
    pub confidence: Option<f32>,
    pub speaker_id: Option<Uuid>,
    pub position: Option<SourcePosition>,
    pub provenance_ref: Option<ContentReference>,
});

impl SourceSpan {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_base(&self.contract_version, &self.observation_origin)?;
        if self.revision == 0 || self.observation_origin.is_empty() {
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
            || self.idempotency_key.len() < 16
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
            || self.engine_version.is_empty()
            || self.reason_code.is_empty()
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
            || self.engine_version.is_empty()
            || self.engine_revision.is_empty()
            || self.public_entry_point.is_empty()
            || self.capability.capability_id.is_empty()
            || self.capability.reason_code.is_empty()
            || self.input_fingerprint.is_empty()
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
        if self.request_revision == 0 || self.failure_code.as_deref().is_some_and(str::is_empty) {
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

/// Validates one ordered event page before merging it into a session history.
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
        if matches!(
            event.event_type,
            TranslationEventType::Completed
                | TranslationEventType::Unsupported
                | TranslationEventType::Suspended
                | TranslationEventType::Failed
                | TranslationEventType::Cancelled
                | TranslationEventType::TimedOut
        ) {
            terminal_seen = true;
        }
        previous = Some(event);
    }
    Ok(())
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
    pub destination: ContentReference,
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
        self.destination.validate()?;
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
    #[error("session fields are incomplete or inconsistent")]
    InvalidSession,
    #[error("source span fields are incomplete or inconsistent")]
    InvalidSourceSpan,
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

    #[test]
    fn tagged_record_round_trips_with_logical_discriminator() {
        let record =
            LogicalRecord::TranslationRequest(request(ProcessingLocation::LocalOnly, None));
        let encoded = serde_json::to_value(&record).unwrap();
        assert_eq!(encoded["record_type"], "TranslationRequest");
        assert_eq!(
            serde_json::from_value::<LogicalRecord>(encoded).unwrap(),
            record
        );
    }

    #[test]
    fn language_tags_are_validated_and_canonicalized() {
        assert_eq!(
            "zh-hant-tw".parse::<LanguageTag>().unwrap().as_str(),
            "zh-Hant-TW"
        );
        assert!("not a tag".parse::<LanguageTag>().is_err());
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
    fn remote_processing_cannot_be_constructed_without_consent_reference() {
        assert_eq!(
            request(ProcessingLocation::RemoteWithConsent, None).validate(),
            Err(ContractError::MissingProcessingConsent)
        );
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
            destination: ContentReference {
                kind: ReferenceKind::Remote,
                key: "recipient:example".into(),
                media_type: None,
                sha256: None,
                size_bytes: None,
            },
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
}
