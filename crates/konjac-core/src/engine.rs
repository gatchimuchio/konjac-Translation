//! Explicit, fail-closed boundary for the selected meaning engine.
//!
//! This module defines the Rust side of the integration contract only. It has
//! no MINIDORA runtime adapter and no alternate translation engine. A runtime
//! adapter may execute only with a permit produced from matching, versioned
//! `SUPPORTED` evidence.

use std::collections::{BTreeSet, HashMap};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    CapabilityEvidence, CapabilityState, EngineExecution, ExecutionMode, ExecutionOutcome,
    LanguageTag, SourceAsset, SourceSpan, TranslationRequest, TranslationResult,
    TranslationResultStatus, TranslationSession, TranslationVariant, VariantStatus,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineIdentity {
    pub name: String,
    pub version: String,
    pub revision: String,
    pub public_entry_point: String,
}

impl EngineIdentity {
    fn validate(&self) -> bool {
        !self.name.is_empty()
            && !self.version.is_empty()
            && self.revision.len() == 40
            && self
                .revision
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            && !self.public_entry_point.is_empty()
    }

    fn matches(&self, evidence: &CapabilityEvidence) -> bool {
        self.version == evidence.engine_version && self.revision == evidence.engine_revision
    }
}

/// Capability is scoped to a language direction, context requirement, and
/// execution mode. A generic "translation supported" flag is insufficient.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityQuery {
    pub source_language: Option<LanguageTag>,
    pub target_language: LanguageTag,
    pub context_required: bool,
    pub execution_mode: ExecutionMode,
}

impl CapabilityQuery {
    pub fn capability_id(&self) -> String {
        let source = self
            .source_language
            .as_ref()
            .map(LanguageTag::as_str)
            .unwrap_or("und");
        let context = if self.context_required {
            "required"
        } else {
            "none"
        };
        let mode = match self.execution_mode {
            ExecutionMode::Batch => "batch",
            ExecutionMode::Streaming => "streaming",
        };
        format!(
            "translation.text.v1|source={source}|target={}|context={context}|mode={mode}",
            self.target_language.as_str()
        )
    }
}

/// Opaque proof that an exact engine build has an audited capability.
/// Fields are private so callers cannot manufacture an execution permit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionPermit {
    engine: EngineIdentity,
    query: CapabilityQuery,
    evidence: CapabilityEvidence,
}

impl ExecutionPermit {
    pub fn engine(&self) -> &EngineIdentity {
        &self.engine
    }

    pub fn capability_id(&self) -> String {
        self.query.capability_id()
    }

    pub fn query(&self) -> &CapabilityQuery {
        &self.query
    }

    pub fn evidence_time(&self) -> DateTime<Utc> {
        self.evidence.observed_at
    }

    fn matches_execution(&self, execution: &EngineExecution) -> bool {
        execution.engine_name == self.engine.name
            && execution.engine_version == self.engine.version
            && execution.engine_revision == self.engine.revision
            && execution.public_entry_point == self.engine.public_entry_point
            && execution.capability == self.evidence
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateStop {
    pub capability_id: String,
    pub state: CapabilityState,
    pub reason_code: String,
}

/// Returns a permit only when the evidence matches the selected build and
/// explicitly says the requested capability is supported. Unknown or
/// mismatched evidence suspends; known missing capability stays unsupported.
pub fn require_capability(
    engine: &EngineIdentity,
    query: &CapabilityQuery,
    evidence: &CapabilityEvidence,
) -> Result<ExecutionPermit, GateStop> {
    let requested_capability = query.capability_id();
    if evidence.validate().is_err()
        || !engine.validate()
        || evidence.capability_id != requested_capability
        || !engine.matches(evidence)
    {
        return Err(GateStop {
            capability_id: requested_capability,
            state: CapabilityState::Suspend,
            reason_code: "CAPABILITY_EVIDENCE_MISMATCH".to_owned(),
        });
    }

    match evidence.state {
        CapabilityState::Supported => Ok(ExecutionPermit {
            engine: engine.clone(),
            query: query.clone(),
            evidence: evidence.clone(),
        }),
        CapabilityState::Unsupported | CapabilityState::Suspend => Err(GateStop {
            capability_id: requested_capability,
            state: evidence.state.clone(),
            reason_code: evidence.reason_code.clone(),
        }),
    }
}

/// Performs the shared input/session checks and exact capability preflight
/// before invoking the single selected engine. Any denied or unknown query
/// stops the whole request; no subset or alternate translator is executed.
pub fn execute_guarded(
    engine: &dyn EnginePort,
    session: &TranslationSession,
    request: &TranslationRequest,
    source_assets: &[SourceAsset],
    source_spans: &[SourceSpan],
) -> Result<EngineRunResult, EnginePortError> {
    request
        .validate_against_session(session, source_assets, source_spans)
        .map_err(map_contract_error)?;

    let context_required = !request.context_refs.is_empty() || !request.terminology_refs.is_empty();
    let queries: BTreeSet<_> = source_spans
        .iter()
        .flat_map(|span| {
            request
                .target_languages
                .iter()
                .map(|target_language| CapabilityQuery {
                    source_language: span.source_language.clone(),
                    target_language: target_language.clone(),
                    context_required,
                    execution_mode: request.execution_mode,
                })
        })
        .collect();
    let mut permits = Vec::with_capacity(queries.len());
    let mut stops = Vec::new();
    for query in &queries {
        let evidence = engine.inspect_capability(query);
        match require_capability(engine.identity(), query, &evidence) {
            Ok(permit) => permits.push(permit),
            Err(stop) => stops.push(stop),
        }
    }
    if !stops.is_empty() {
        return Err(EnginePortError::CapabilityStopped(stops));
    }

    let run = engine.execute(request, source_spans, &permits)?;
    run.validate_for(session, request, source_assets, source_spans, &permits)?;
    Ok(run)
}

fn map_contract_error(error: crate::ContractError) -> EnginePortError {
    match error {
        crate::ContractError::SourceSpanReferenceMismatch
        | crate::ContractError::SourceAssetReferenceMismatch
        | crate::ContractError::InvalidSourceSpan => EnginePortError::SourceSpanReferenceMismatch,
        crate::ContractError::SessionReferenceMismatch
        | crate::ContractError::SessionNotActive
        | crate::ContractError::SessionPolicyMismatch => EnginePortError::SessionReferenceMismatch,
        _ => EnginePortError::InvalidRequest,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CancellationObservation {
    Requested,
    Stopped,
    Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EngineRunResult {
    pub result: TranslationResult,
    pub executions: Vec<EngineExecution>,
    pub variants: Vec<TranslationVariant>,
}

impl EngineRunResult {
    pub fn validate_for(
        &self,
        session: &TranslationSession,
        request: &TranslationRequest,
        source_assets: &[SourceAsset],
        source_spans: &[SourceSpan],
        permits: &[ExecutionPermit],
    ) -> Result<(), EnginePortError> {
        request
            .validate_against_session(session, source_assets, source_spans)
            .map_err(map_contract_error)?;
        self.result
            .validate()
            .map_err(|_| EnginePortError::InvalidResult)?;
        if self.result.request_id != request.record_id
            || self.result.request_revision != request.revision
        {
            return Err(EnginePortError::RequestReferenceMismatch);
        }

        let mut spans = HashMap::new();
        for span in source_spans {
            span.validate()
                .map_err(|_| EnginePortError::InvalidSourceSpan)?;
            if span.session_id != request.session_id
                || span.revision != request.input_revision
                || spans.insert(span.record_id, span).is_some()
            {
                return Err(EnginePortError::SourceSpanReferenceMismatch);
            }
        }
        if spans.keys().copied().collect::<BTreeSet<_>>()
            != request.source_span_ids.iter().copied().collect()
        {
            return Err(EnginePortError::SourceSpanReferenceMismatch);
        }

        let context_required =
            !request.context_refs.is_empty() || !request.terminology_refs.is_empty();
        let expected_queries: BTreeSet<_> = source_spans
            .iter()
            .flat_map(|span| {
                request
                    .target_languages
                    .iter()
                    .map(|target_language| CapabilityQuery {
                        source_language: span.source_language.clone(),
                        target_language: target_language.clone(),
                        context_required,
                        execution_mode: request.execution_mode,
                    })
            })
            .collect();
        let mut actual_queries = BTreeSet::new();
        let mut selected_engine: Option<&EngineIdentity> = None;
        for permit in permits {
            if !actual_queries.insert(permit.query.clone()) {
                return Err(EnginePortError::CapabilityPermitMismatch);
            }
            if selected_engine.is_some_and(|engine| engine != &permit.engine) {
                return Err(EnginePortError::CapabilityPermitMismatch);
            }
            selected_engine = Some(&permit.engine);
        }
        if actual_queries != expected_queries {
            return Err(EnginePortError::CapabilityPermitMismatch);
        }
        let permits_by_id: HashMap<_, _> = permits
            .iter()
            .map(|permit| (permit.capability_id(), permit))
            .collect();

        let mut executions = HashMap::new();
        for execution in &self.executions {
            execution
                .validate()
                .map_err(|_| EnginePortError::InvalidExecution)?;
            if execution.request_id != request.record_id
                || executions.insert(execution.record_id, execution).is_some()
            {
                return Err(EnginePortError::RequestReferenceMismatch);
            }
            let permit = permits_by_id
                .get(&execution.capability.capability_id)
                .ok_or(EnginePortError::CapabilityPermitMismatch)?;
            if !permit.matches_execution(execution)
                || execution.outcome != ExecutionOutcome::Completed
                    && !matches!(
                        execution.outcome,
                        ExecutionOutcome::Failed
                            | ExecutionOutcome::Cancelled
                            | ExecutionOutcome::TimedOut
                    )
            {
                return Err(EnginePortError::CapabilityPermitMismatch);
            }
        }

        let result_execution_ids: BTreeSet<_> =
            self.result.engine_execution_ids.iter().copied().collect();
        if result_execution_ids.len() != self.result.engine_execution_ids.len()
            || result_execution_ids != executions.keys().copied().collect()
        {
            return Err(EnginePortError::ExecutionReferenceMismatch);
        }

        let mut actual_pairs = BTreeSet::new();
        let mut final_count = 0;
        for variant in &self.variants {
            variant
                .validate()
                .map_err(|_| EnginePortError::InvalidVariant)?;
            if !request.source_span_ids.contains(&variant.source_span_id)
                || !request.target_languages.contains(&variant.target_language)
                || !actual_pairs.insert((variant.source_span_id, variant.target_language.clone()))
            {
                return Err(EnginePortError::VariantOutsideRequest);
            }
            if variant.status == VariantStatus::Final {
                final_count += 1;
            }
            if let Some(execution_id) = variant.engine_execution_id {
                let execution = executions
                    .get(&execution_id)
                    .ok_or(EnginePortError::ExecutionReferenceMismatch)?;
                variant
                    .validate_against(execution)
                    .map_err(|_| EnginePortError::InvalidVariant)?;
                let span = spans
                    .get(&variant.source_span_id)
                    .ok_or(EnginePortError::SourceSpanReferenceMismatch)?;
                let query = CapabilityQuery {
                    source_language: span.source_language.clone(),
                    target_language: variant.target_language.clone(),
                    context_required,
                    execution_mode: request.execution_mode,
                };
                if execution.capability.capability_id != query.capability_id() {
                    return Err(EnginePortError::CapabilityPermitMismatch);
                }
            }
        }

        let result_variant_ids: BTreeSet<_> = self.result.variant_ids.iter().copied().collect();
        if result_variant_ids.len() != self.result.variant_ids.len()
            || result_variant_ids
                != self
                    .variants
                    .iter()
                    .map(|variant| variant.record_id)
                    .collect()
        {
            return Err(EnginePortError::VariantReferenceMismatch);
        }

        match &self.result.status {
            TranslationResultStatus::Completed | TranslationResultStatus::Partial => {
                let expected_pairs: BTreeSet<_> = request
                    .source_span_ids
                    .iter()
                    .flat_map(|span_id| {
                        request
                            .target_languages
                            .iter()
                            .map(move |language| (*span_id, language.clone()))
                    })
                    .collect();
                if actual_pairs != expected_pairs
                    || self
                        .variants
                        .iter()
                        .any(|variant| variant.status == VariantStatus::Provisional)
                {
                    return Err(EnginePortError::IncompleteResultCoverage);
                }
                if (matches!(&self.result.status, TranslationResultStatus::Completed)
                    && final_count != self.variants.len())
                    || (matches!(&self.result.status, TranslationResultStatus::Partial)
                        && (final_count == 0 || final_count == self.variants.len()))
                {
                    return Err(EnginePortError::InvalidResult);
                }
            }
            TranslationResultStatus::Unsupported
            | TranslationResultStatus::Suspend
            | TranslationResultStatus::Failed
            | TranslationResultStatus::Cancelled
            | TranslationResultStatus::TimedOut
                if !self.variants.is_empty() =>
            {
                return Err(EnginePortError::UnsuccessfulRunHasVariants);
            }
            _ => {}
        }
        Ok(())
    }
}

/// The single selected semantic engine boundary. Implementations must not
/// substitute another engine or treat cancellation acceptance as completion.
pub trait EnginePort: Send + Sync {
    fn identity(&self) -> &EngineIdentity;

    fn inspect_capability(&self, query: &CapabilityQuery) -> CapabilityEvidence;

    fn execute(
        &self,
        request: &TranslationRequest,
        source_spans: &[SourceSpan],
        permits: &[ExecutionPermit],
    ) -> Result<EngineRunResult, EnginePortError>;

    fn request_cancel(
        &self,
        execution_id: Uuid,
    ) -> Result<CancellationObservation, EnginePortError>;
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EnginePortError {
    #[error("request is invalid")]
    InvalidRequest,
    #[error("source span is invalid")]
    InvalidSourceSpan,
    #[error("source spans do not match the request revision or session")]
    SourceSpanReferenceMismatch,
    #[error("request does not match the active session revision and policy")]
    SessionReferenceMismatch,
    #[error("capability permits do not exactly match the requested capability set")]
    CapabilityPermitMismatch,
    #[error("one or more exact capability queries denied execution: {0:?}")]
    CapabilityStopped(Vec<GateStop>),
    #[error("execution evidence is invalid")]
    InvalidExecution,
    #[error("translation result is invalid for its request")]
    InvalidResult,
    #[error("engine execution refers to a different request")]
    RequestReferenceMismatch,
    #[error("result refers to an execution that is missing or duplicated")]
    ExecutionReferenceMismatch,
    #[error("result refers to a variant that is missing or duplicated")]
    VariantReferenceMismatch,
    #[error("unsuccessful engine run returned translation variants")]
    UnsuccessfulRunHasVariants,
    #[error("engine returned a variant outside the requested source/target set")]
    VariantOutsideRequest,
    #[error("terminal result does not cover every requested source/target pair")]
    IncompleteResultCoverage,
    #[error("engine variant and execution evidence disagree")]
    InvalidVariant,
    #[error("selected engine runtime operation failed")]
    RuntimeFailure,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CapabilityState, InputOrigin, MediaKind, Participant, ParticipantRole, ProcessingLocation,
        RequestedOutput, SessionStatus, SpanPayload, TranslationResultStatus,
    };
    use chrono::TimeZone;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 9, hour, 0, 0).unwrap()
    }

    fn engine() -> EngineIdentity {
        EngineIdentity {
            name: "MINIDORA-BETA".into(),
            version: "0.6.0b1".into(),
            revision: "fe5822de63644bdaea9208a98046f71684dffae2".into(),
            public_entry_point: "minidora.engine_beta.ミニドラβ実行機.実行".into(),
        }
    }

    fn query(target: &str) -> CapabilityQuery {
        CapabilityQuery {
            source_language: Some("ja".parse().unwrap()),
            target_language: target.parse().unwrap(),
            context_required: false,
            execution_mode: ExecutionMode::Batch,
        }
    }

    fn evidence(state: CapabilityState, query: &CapabilityQuery) -> CapabilityEvidence {
        CapabilityEvidence {
            capability_id: query.capability_id(),
            state,
            engine_version: "0.6.0b1".into(),
            engine_revision: "fe5822de63644bdaea9208a98046f71684dffae2".into(),
            reason_code: "NO_PUBLIC_TRANSLATION_OPERATION".into(),
            observed_at: at(1),
        }
    }

    struct UnsupportedPort {
        identity: EngineIdentity,
    }

    impl EnginePort for UnsupportedPort {
        fn identity(&self) -> &EngineIdentity {
            &self.identity
        }

        fn inspect_capability(&self, query: &CapabilityQuery) -> CapabilityEvidence {
            let unsupported = query.target_language.as_str() == "en";
            CapabilityEvidence {
                capability_id: query.capability_id(),
                state: if unsupported {
                    CapabilityState::Unsupported
                } else {
                    CapabilityState::Supported
                },
                engine_version: self.identity.version.clone(),
                engine_revision: self.identity.revision.clone(),
                reason_code: if unsupported {
                    "NO_PUBLIC_TRANSLATION_OPERATION".into()
                } else {
                    "TEST_FIXTURE_SUPPORTED".into()
                },
                observed_at: at(1),
            }
        }

        fn execute(
            &self,
            _request: &TranslationRequest,
            _source_spans: &[SourceSpan],
            _permits: &[ExecutionPermit],
        ) -> Result<EngineRunResult, EnginePortError> {
            panic!("execution must not run after unsupported capability evidence")
        }

        fn request_cancel(
            &self,
            _execution_id: Uuid,
        ) -> Result<CancellationObservation, EnginePortError> {
            Ok(CancellationObservation::Unknown)
        }
    }

    #[test]
    fn audited_missing_translation_operation_stays_unsupported() {
        let query = query("en");
        let stop = require_capability(
            &engine(),
            &query,
            &evidence(CapabilityState::Unsupported, &query),
        )
        .unwrap_err();
        assert_eq!(stop.state, CapabilityState::Unsupported);
        assert_eq!(stop.reason_code, "NO_PUBLIC_TRANSLATION_OPERATION");
    }

    #[test]
    fn guarded_run_stops_before_engine_invocation_when_capability_is_unsupported() {
        let (mut session, mut request, asset, span, _, _) = contract_fixture();
        session.target_languages.push("fr".parse().unwrap());
        request.target_languages.push("fr".parse().unwrap());
        let port = UnsupportedPort { identity: engine() };
        let expected_query = query("en");
        assert_eq!(
            execute_guarded(&port, &session, &request, &[asset], &[span]),
            Err(EnginePortError::CapabilityStopped(vec![GateStop {
                capability_id: expected_query.capability_id(),
                state: CapabilityState::Unsupported,
                reason_code: "NO_PUBLIC_TRANSLATION_OPERATION".into(),
            }]))
        );
    }

    #[test]
    fn unknown_or_wrong_version_evidence_suspends_without_permit() {
        let query = query("en");
        let mut unknown = evidence(CapabilityState::Supported, &query);
        unknown.engine_version = "0.7.0".into();
        let stop = require_capability(&engine(), &query, &unknown).unwrap_err();
        assert_eq!(stop.state, CapabilityState::Suspend);
        assert_eq!(stop.reason_code, "CAPABILITY_EVIDENCE_MISMATCH");
    }

    #[test]
    fn only_matching_supported_evidence_creates_an_execution_permit() {
        let query = query("en");
        let permit = require_capability(
            &engine(),
            &query,
            &evidence(CapabilityState::Supported, &query),
        )
        .unwrap();
        assert_eq!(permit.engine(), &engine());
        assert_eq!(permit.query(), &query);
        assert_eq!(permit.capability_id(), query.capability_id());
    }

    fn supported_permit(query: &CapabilityQuery) -> ExecutionPermit {
        require_capability(
            &engine(),
            query,
            &evidence(CapabilityState::Supported, query),
        )
        .unwrap()
    }

    fn contract_fixture() -> (
        TranslationSession,
        TranslationRequest,
        SourceAsset,
        SourceSpan,
        Vec<ExecutionPermit>,
        EngineRunResult,
    ) {
        let request = TranslationRequest {
            contract_version: crate::LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(20),
            created_at: at(1),
            session_id: Uuid::from_u128(19),
            session_revision: 1,
            revision: 1,
            source_span_ids: vec![Uuid::from_u128(21)],
            input_revision: 1,
            target_languages: vec!["en".parse().unwrap()],
            context_refs: vec![],
            terminology_refs: vec![],
            requested_outputs: vec![RequestedOutput::Text],
            execution_mode: ExecutionMode::Batch,
            processing_location: ProcessingLocation::LocalOnly,
            processing_consent_id: None,
            idempotency_key: "request-contract-fixture-01".into(),
            attempt: 1,
            deadline: None,
            cancel_requested_at: None,
        };
        let session = TranslationSession {
            contract_version: crate::LOGICAL_CONTRACT_VERSION.into(),
            record_id: request.session_id,
            created_at: at(1),
            revision: request.session_revision,
            status: SessionStatus::Active,
            source_language: Some("ja".parse().unwrap()),
            target_languages: request.target_languages.clone(),
            participants: vec![Participant {
                participant_id: Uuid::from_u128(25),
                role: ParticipantRole::Owner,
                preferred_language: Some("ja".parse().unwrap()),
                consent_grant_ids: vec![],
            }],
            context_refs: request.context_refs.clone(),
            terminology_refs: request.terminology_refs.clone(),
            processing_location: request.processing_location.clone(),
            consent_grant_ids: vec![],
            retention: crate::RetentionPolicy::Ephemeral,
        };
        let span = SourceSpan {
            contract_version: crate::LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(21),
            created_at: at(1),
            asset_id: Uuid::from_u128(18),
            session_id: request.session_id,
            revision: request.input_revision,
            payload: SpanPayload::Text {
                text: "synthetic fixture input".into(),
            },
            source_language: Some("ja".parse().unwrap()),
            observation_origin: "typed_fixture".into(),
            confidence: None,
            speaker_id: None,
            position: None,
            provenance_ref: None,
        };
        let asset = SourceAsset {
            contract_version: crate::LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(18),
            created_at: at(1),
            media_kind: MediaKind::Text,
            origin: InputOrigin::UserInput,
            media_type: "text/plain".into(),
            content_ref: None,
            original_fingerprint: None,
            owner_ref: None,
            rights_ref: None,
            capture_consent_id: None,
            time_basis: None,
            retention: crate::RetentionPolicy::Ephemeral,
            metadata: std::collections::BTreeMap::new(),
        };
        let en_query = query("en");
        let permits = vec![supported_permit(&en_query)];
        let execution = EngineExecution {
            contract_version: crate::LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(23),
            created_at: at(1),
            request_id: request.record_id,
            engine_name: "MINIDORA-BETA".into(),
            engine_version: "0.6.0b1".into(),
            engine_revision: engine().revision,
            public_entry_point: engine().public_entry_point,
            capability: evidence(CapabilityState::Supported, &en_query),
            input_fingerprint: "0000000000000000000000000000000000000000000000000000000000000000"
                .into(),
            response_fingerprint: Some(
                "1111111111111111111111111111111111111111111111111111111111111111".into(),
            ),
            started_at: at(1),
            finished_at: Some(at(2)),
            outcome: ExecutionOutcome::Completed,
            diagnostic_code: None,
            trace_ref: None,
        };
        let variant = TranslationVariant {
            contract_version: crate::LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(24),
            created_at: at(2),
            request_id: request.record_id,
            source_span_id: Uuid::from_u128(21),
            target_language: "en".parse().unwrap(),
            revision: 1,
            status: VariantStatus::Final,
            text: Some("fixture output, not a translation claim".into()),
            engine_execution_id: Some(execution.record_id),
            provenance_ref: None,
            speaker_id: None,
            position: None,
            produced_at: at(2),
        };
        let result = TranslationResult {
            contract_version: crate::LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(22),
            created_at: at(1),
            request_id: request.record_id,
            request_revision: 1,
            status: TranslationResultStatus::Completed,
            variant_ids: vec![variant.record_id],
            engine_execution_ids: vec![execution.record_id],
            failure_code: None,
            completed_at: Some(at(2)),
        };
        (
            session,
            request,
            asset,
            span,
            permits,
            EngineRunResult {
                result,
                executions: vec![execution],
                variants: vec![variant],
            },
        )
    }

    #[test]
    fn terminal_result_must_cover_each_requested_source_and_target_pair() {
        let (mut session, mut request, asset, span, mut permits, result) = contract_fixture();
        assert_eq!(
            result.validate_for(
                &session,
                &request,
                std::slice::from_ref(&asset),
                std::slice::from_ref(&span),
                &permits
            ),
            Ok(())
        );
        request.target_languages.push("fr".parse().unwrap());
        session.target_languages.push("fr".parse().unwrap());
        permits.push(supported_permit(&query("fr")));
        assert_eq!(
            result.validate_for(&session, &request, &[asset], &[span], &permits),
            Err(EnginePortError::IncompleteResultCoverage)
        );
    }

    #[test]
    fn result_revision_must_match_the_request_revision() {
        let (session, mut request, asset, span, permits, result) = contract_fixture();
        request.revision = 2;
        assert_eq!(
            result.validate_for(&session, &request, &[asset], &[span], &permits),
            Err(EnginePortError::RequestReferenceMismatch)
        );
        let (session, mut request, asset, span, permits, result) = contract_fixture();
        request.session_revision = 2;
        assert_eq!(
            result.validate_for(&session, &request, &[asset], &[span], &permits),
            Err(EnginePortError::SessionReferenceMismatch)
        );
        let (mut session, request, asset, span, permits, result) = contract_fixture();
        session.status = SessionStatus::Paused;
        assert_eq!(
            result.validate_for(&session, &request, &[asset], &[span], &permits),
            Err(EnginePortError::SessionReferenceMismatch)
        );
    }

    #[test]
    fn every_source_span_must_resolve_to_its_source_asset() {
        let (session, request, mut asset, span, permits, result) = contract_fixture();
        asset.record_id = Uuid::from_u128(99);
        assert_eq!(
            result.validate_for(&session, &request, &[asset], &[span], &permits),
            Err(EnginePortError::SourceSpanReferenceMismatch)
        );
    }

    #[test]
    fn permits_must_match_source_language_context_and_execution_mode() {
        let (session, mut streaming_request, asset, span, permits, result) = contract_fixture();
        streaming_request.execution_mode = ExecutionMode::Streaming;
        assert_eq!(
            result.validate_for(
                &session,
                &streaming_request,
                std::slice::from_ref(&asset),
                std::slice::from_ref(&span),
                &permits
            ),
            Err(EnginePortError::CapabilityPermitMismatch)
        );

        let (session, mut context_request, asset, span, permits, result) = contract_fixture();
        context_request.context_refs.push(crate::ContentReference {
            kind: crate::ReferenceKind::Local,
            key: "fixture-context".into(),
            media_type: None,
            sha256: None,
            size_bytes: None,
        });
        assert_eq!(
            result.validate_for(
                &session,
                &context_request,
                std::slice::from_ref(&asset),
                std::slice::from_ref(&span),
                &permits
            ),
            Err(EnginePortError::CapabilityPermitMismatch)
        );

        let (session, request, asset, mut span, permits, result) = contract_fixture();
        span.source_language = Some("fr".parse().unwrap());
        assert_eq!(
            result.validate_for(&session, &request, &[asset], &[span], &permits),
            Err(EnginePortError::CapabilityPermitMismatch)
        );
    }
}
