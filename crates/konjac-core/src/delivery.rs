//! External delivery authorization preflight.
//!
//! This module creates artifact-bound permits only after checking the named
//! grant. It does not perform delivery or provide durable/atomic authorization;
//! adapters must revalidate immediately before side effects.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::consent::ConsentRegistry;
use crate::engine::AuthorizationContext;
use crate::{ConsentScope, DeliveryArtifact, DeliveryStatus};

#[derive(Clone, Debug, PartialEq)]
pub struct DeliveryPermit {
    artifact: DeliveryArtifact,
    principal_id: String,
}

impl DeliveryPermit {
    /// Rechecks that this permit still names the exact artifact and that its
    /// delivery grant remains valid at the point of use.
    pub fn validate_for(
        &self,
        artifact: &DeliveryArtifact,
        authorization: &AuthorizationContext<'_>,
        now: DateTime<Utc>,
    ) -> Result<(), DeliveryAuthorizationError> {
        if artifact != &self.artifact || artifact.status != DeliveryStatus::Authorized {
            return Err(DeliveryAuthorizationError::PermitMismatch);
        }
        if authorization.principal_id != self.principal_id {
            return Err(DeliveryAuthorizationError::AuthorizationDenied);
        }
        let grant_id = artifact
            .delivery_consent_id
            .ok_or(DeliveryAuthorizationError::AuthorizationDenied)?;
        if !ConsentRegistry::new(authorization.grants, authorization.revocations).authorizes(
            grant_id,
            authorization.principal_id,
            ConsentScope::Deliver,
            artifact.record_id,
            now,
        ) {
            return Err(DeliveryAuthorizationError::AuthorizationDenied);
        }
        Ok(())
    }
}

/// Creates a permit for an artifact whose state machine has recorded the
/// separate delivery authorization. Only `Authorized` artifacts can proceed;
/// `Prepared`, terminal and uncertain artifacts must be handled elsewhere.
pub fn authorize_delivery_at(
    artifact: &DeliveryArtifact,
    authorization: &AuthorizationContext<'_>,
    now: DateTime<Utc>,
) -> Result<DeliveryPermit, DeliveryAuthorizationError> {
    artifact
        .validate()
        .map_err(|_| DeliveryAuthorizationError::InvalidArtifact)?;
    if artifact.status != DeliveryStatus::Authorized {
        return Err(DeliveryAuthorizationError::ArtifactNotAuthorized);
    }
    let grant_id = artifact
        .delivery_consent_id
        .ok_or(DeliveryAuthorizationError::AuthorizationDenied)?;
    if !ConsentRegistry::new(authorization.grants, authorization.revocations).authorizes(
        grant_id,
        authorization.principal_id,
        ConsentScope::Deliver,
        artifact.record_id,
        now,
    ) {
        return Err(DeliveryAuthorizationError::AuthorizationDenied);
    }

    Ok(DeliveryPermit {
        artifact: artifact.clone(),
        principal_id: authorization.principal_id.to_owned(),
    })
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum DeliveryAuthorizationError {
    #[error("delivery artifact is invalid")]
    InvalidArtifact,
    #[error("delivery artifact is not in the authorized state")]
    ArtifactNotAuthorized,
    #[error("delivery consent does not authorize this principal and artifact")]
    AuthorizationDenied,
    #[error("delivery permit does not match the exact authorized artifact")]
    PermitMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConsentGrant, ConsentRevocation, LOGICAL_CONTRACT_VERSION, ReferenceKind};
    use chrono::TimeZone;
    use uuid::Uuid;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 9, hour, 0, 0).unwrap()
    }

    fn artifact(grant_id: Uuid) -> DeliveryArtifact {
        DeliveryArtifact {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(50),
            created_at: at(1),
            variant_id: Uuid::from_u128(51),
            format: crate::ArtifactFormat::Text,
            destination: crate::ContentReference {
                kind: ReferenceKind::Remote,
                key: "recipient:example".into(),
                media_type: Some("text/plain".into()),
                sha256: None,
                size_bytes: None,
            },
            delivery_consent_id: Some(grant_id),
            status: DeliveryStatus::Authorized,
            delivered_at: None,
            audit_id: Uuid::from_u128(52),
        }
    }

    fn grant(grant_id: Uuid, scope: ConsentScope, subject_id: Uuid) -> ConsentGrant {
        ConsentGrant {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: grant_id,
            created_at: at(1),
            principal_id: "authenticated-owner".into(),
            scope,
            subject_ids: vec![subject_id],
            granted_at: at(1),
            expires_at: Some(at(4)),
            evidence_ref: None,
        }
    }

    #[test]
    fn delivery_permit_binds_active_grant_to_exact_artifact_and_principal() {
        let grant_id = Uuid::from_u128(53);
        let artifact = artifact(grant_id);
        let grants = [grant(grant_id, ConsentScope::Deliver, artifact.record_id)];
        let authorization = AuthorizationContext {
            principal_id: "authenticated-owner",
            grants: &grants,
            revocations: &[],
        };

        let permit = authorize_delivery_at(&artifact, &authorization, at(2)).unwrap();
        assert!(
            permit
                .validate_for(&artifact, &authorization, at(2))
                .is_ok()
        );

        let mut changed_artifact = artifact.clone();
        changed_artifact.destination.key = "recipient:other".into();
        assert_eq!(
            permit.validate_for(&changed_artifact, &authorization, at(2)),
            Err(DeliveryAuthorizationError::PermitMismatch)
        );

        let wrong_principal = AuthorizationContext {
            principal_id: "different-owner",
            grants: &grants,
            revocations: &[],
        };
        assert_eq!(
            permit.validate_for(&artifact, &wrong_principal, at(2)),
            Err(DeliveryAuthorizationError::AuthorizationDenied)
        );

        assert_eq!(
            permit.validate_for(&artifact, &authorization, at(4)),
            Err(DeliveryAuthorizationError::AuthorizationDenied)
        );

        let revocations = [ConsentRevocation {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(57),
            created_at: at(3),
            grant_id,
            revoked_by: "authenticated-owner".into(),
            revoked_at: at(3),
            reason_code: "USER_REVOKED".into(),
        }];
        let revoked_authorization = AuthorizationContext {
            principal_id: "authenticated-owner",
            grants: &grants,
            revocations: &revocations,
        };
        assert_eq!(
            permit.validate_for(&artifact, &revoked_authorization, at(3)),
            Err(DeliveryAuthorizationError::AuthorizationDenied)
        );
    }

    #[test]
    fn delivery_preflight_rejects_wrong_scope_expired_and_revoked_grants() {
        let grant_id = Uuid::from_u128(54);
        let artifact = artifact(grant_id);
        let wrong_scope_grants = [grant(
            grant_id,
            ConsentScope::ProcessRemote,
            artifact.record_id,
        )];
        let wrong_scope = AuthorizationContext {
            principal_id: "authenticated-owner",
            grants: &wrong_scope_grants,
            revocations: &[],
        };
        assert_eq!(
            authorize_delivery_at(&artifact, &wrong_scope, at(2)),
            Err(DeliveryAuthorizationError::AuthorizationDenied)
        );

        let valid_grants = [grant(grant_id, ConsentScope::Deliver, artifact.record_id)];
        let expired = AuthorizationContext {
            principal_id: "authenticated-owner",
            grants: &valid_grants,
            revocations: &[],
        };
        assert_eq!(
            authorize_delivery_at(&artifact, &expired, at(4)),
            Err(DeliveryAuthorizationError::AuthorizationDenied)
        );

        let revocations = [ConsentRevocation {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(55),
            created_at: at(3),
            grant_id,
            revoked_by: "authenticated-owner".into(),
            revoked_at: at(3),
            reason_code: "USER_REVOKED".into(),
        }];
        let revoked = AuthorizationContext {
            principal_id: "authenticated-owner",
            grants: &valid_grants,
            revocations: &revocations,
        };
        assert_eq!(
            authorize_delivery_at(&artifact, &revoked, at(3)),
            Err(DeliveryAuthorizationError::AuthorizationDenied)
        );
    }

    #[test]
    fn delivery_preflight_requires_the_authorized_state_and_artifact_subject() {
        let grant_id = Uuid::from_u128(56);
        let mut artifact = artifact(grant_id);
        let grants = [grant(grant_id, ConsentScope::Deliver, Uuid::from_u128(999))];
        let authorization = AuthorizationContext {
            principal_id: "authenticated-owner",
            grants: &grants,
            revocations: &[],
        };
        assert_eq!(
            authorize_delivery_at(&artifact, &authorization, at(2)),
            Err(DeliveryAuthorizationError::AuthorizationDenied)
        );

        artifact.status = DeliveryStatus::Prepared;
        assert_eq!(
            authorize_delivery_at(&artifact, &authorization, at(2)),
            Err(DeliveryAuthorizationError::ArtifactNotAuthorized)
        );
    }
}
