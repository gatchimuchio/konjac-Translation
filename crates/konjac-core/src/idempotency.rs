//! Atomic request-key claims.
//!
//! `InMemoryIdempotencyLedger` prevents concurrent duplicate admission inside
//! one process. It is not durable and must not be used as the production
//! ledger; a persistent implementation must preserve this module's atomic
//! `(session_id, idempotency_key)` and exact-request semantics.

use std::collections::HashMap;
use std::sync::Mutex;

use thiserror::Error;
use uuid::Uuid;

use crate::TranslationRequest;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdempotencyClaim {
    /// This exact request body reserved the key for the first time.
    Reserved,
    /// The exact request body already reserved the key; do not execute it again.
    Replay,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum IdempotencyError {
    #[error("request is invalid")]
    InvalidRequest,
    #[error("idempotency key is already bound to a different request body or revision")]
    Conflict,
    #[error("idempotency ledger is unavailable")]
    Unavailable,
}

/// Storage implementations must atomically reserve `(session_id,
/// idempotency_key)`. The same exact request body returns `Replay`; any
/// difference, including request ID, revision, attempt or inputs, returns
/// `Conflict`. A production implementation must persist claims and expose the
/// existing request's state to the service layer.
pub trait RequestIdempotencyStore: Send + Sync {
    fn claim(&self, request: &TranslationRequest) -> Result<IdempotencyClaim, IdempotencyError>;
}

#[derive(Default)]
pub struct InMemoryIdempotencyLedger {
    claims: Mutex<HashMap<(Uuid, String), TranslationRequest>>,
}

impl RequestIdempotencyStore for InMemoryIdempotencyLedger {
    fn claim(&self, request: &TranslationRequest) -> Result<IdempotencyClaim, IdempotencyError> {
        request
            .validate()
            .map_err(|_| IdempotencyError::InvalidRequest)?;

        let key = (request.session_id, request.idempotency_key.clone());
        let mut claims = self
            .claims
            .lock()
            .map_err(|_| IdempotencyError::Unavailable)?;
        match claims.get(&key) {
            Some(existing) if existing == request => Ok(IdempotencyClaim::Replay),
            Some(_) => Err(IdempotencyError::Conflict),
            None => {
                claims.insert(key, request.clone());
                Ok(IdempotencyClaim::Reserved)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExecutionMode, LOGICAL_CONTRACT_VERSION, ProcessingLocation, RequestedOutput};
    use chrono::{DateTime, TimeZone, Utc};
    use std::sync::{Arc, Barrier};
    use std::thread;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 9, hour, 0, 0).unwrap()
    }

    fn request() -> TranslationRequest {
        TranslationRequest {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(60),
            created_at: at(1),
            session_id: Uuid::from_u128(61),
            session_revision: 1,
            revision: 1,
            source_span_ids: vec![Uuid::from_u128(62)],
            input_revision: 1,
            target_languages: vec!["en".parse().unwrap()],
            context_refs: vec![],
            terminology_refs: vec![],
            requested_outputs: vec![RequestedOutput::Text],
            execution_mode: ExecutionMode::Batch,
            processing_location: ProcessingLocation::LocalOnly,
            processing_consent_id: None,
            idempotency_key: "session-request-key-01".into(),
            attempt: 1,
            deadline: None,
            cancel_requested_at: None,
        }
    }

    #[test]
    fn identical_request_replays_but_changed_body_or_revision_conflicts() {
        let ledger = InMemoryIdempotencyLedger::default();
        let request = request();

        assert_eq!(ledger.claim(&request), Ok(IdempotencyClaim::Reserved));
        assert_eq!(ledger.claim(&request), Ok(IdempotencyClaim::Replay));

        let mut changed_body = request.clone();
        changed_body.target_languages = vec!["fr".parse().unwrap()];
        assert_eq!(ledger.claim(&changed_body), Err(IdempotencyError::Conflict));

        let mut changed_revision = request.clone();
        changed_revision.revision += 1;
        assert_eq!(
            ledger.claim(&changed_revision),
            Err(IdempotencyError::Conflict)
        );

        let mut changed_attempt = request;
        changed_attempt.attempt += 1;
        assert_eq!(
            ledger.claim(&changed_attempt),
            Err(IdempotencyError::Conflict)
        );
    }

    #[test]
    fn idempotency_key_is_scoped_to_a_session_and_invalid_requests_do_not_claim() {
        let ledger = InMemoryIdempotencyLedger::default();
        let mut invalid = request();
        invalid.idempotency_key = "short".into();
        assert_eq!(
            ledger.claim(&invalid),
            Err(IdempotencyError::InvalidRequest)
        );

        let first = request();
        assert_eq!(ledger.claim(&first), Ok(IdempotencyClaim::Reserved));

        let mut other_session = first.clone();
        other_session.session_id = Uuid::from_u128(63);
        assert_eq!(ledger.claim(&other_session), Ok(IdempotencyClaim::Reserved));
    }

    #[test]
    fn concurrent_identical_claims_reserve_exactly_once() {
        const WORKERS: usize = 12;
        let ledger = Arc::new(InMemoryIdempotencyLedger::default());
        let barrier = Arc::new(Barrier::new(WORKERS));
        let request = request();
        let workers: Vec<_> = (0..WORKERS)
            .map(|_| {
                let ledger = Arc::clone(&ledger);
                let barrier = Arc::clone(&barrier);
                let request = request.clone();
                thread::spawn(move || {
                    barrier.wait();
                    ledger.claim(&request).unwrap()
                })
            })
            .collect();

        let outcomes: Vec<_> = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect();
        assert_eq!(
            outcomes
                .iter()
                .filter(|claim| **claim == IdempotencyClaim::Reserved)
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|claim| **claim == IdempotencyClaim::Replay)
                .count(),
            WORKERS - 1
        );
    }
}
