//! Pure consent evaluation. Durable storage and OS permission checks remain
//! separate responsibilities and are not inferred from this registry.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{ConsentGrant, ConsentRevocation, ConsentScope};

pub struct ConsentRegistry<'a> {
    grants: &'a [ConsentGrant],
    revocations: &'a [ConsentRevocation],
}

impl<'a> ConsentRegistry<'a> {
    pub fn new(grants: &'a [ConsentGrant], revocations: &'a [ConsentRevocation]) -> Self {
        Self {
            grants,
            revocations,
        }
    }

    /// Checks one named grant against its exact principal, scope, subject and
    /// time. An invalid matching grant or a recorded effective revocation
    /// fails closed. OS permission and resource ownership must be checked
    /// independently by the corresponding adapter.
    pub fn authorizes(
        &self,
        grant_id: Uuid,
        principal_id: &str,
        scope: ConsentScope,
        subject_id: Uuid,
        at: DateTime<Utc>,
    ) -> bool {
        let Some(grant) = self.grants.iter().find(|grant| grant.record_id == grant_id) else {
            return false;
        };
        if grant.validate().is_err()
            || grant.principal_id != principal_id
            || grant.scope != scope
            || !grant.subject_ids.contains(&subject_id)
            || grant.granted_at > at
            || grant.expires_at.is_some_and(|expires| expires <= at)
        {
            return false;
        }

        !self
            .revocations
            .iter()
            .any(|revocation| revocation.grant_id == grant_id && revocation.revoked_at <= at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConsentScope, LOGICAL_CONTRACT_VERSION};
    use chrono::TimeZone;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 9, hour, 0, 0).unwrap()
    }

    fn grant(scope: ConsentScope, expires_at: Option<DateTime<Utc>>) -> ConsentGrant {
        ConsentGrant {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(1),
            created_at: at(1),
            principal_id: "participant-1".into(),
            scope,
            subject_ids: vec![Uuid::from_u128(2)],
            granted_at: at(1),
            expires_at,
            evidence_ref: None,
        }
    }

    #[test]
    fn a_grant_is_scoped_to_its_principal_operation_subject_and_time() {
        let grants = [grant(ConsentScope::ProcessRemote, Some(at(5)))];
        let registry = ConsentRegistry::new(&grants, &[]);
        assert!(registry.authorizes(
            Uuid::from_u128(1),
            "participant-1",
            ConsentScope::ProcessRemote,
            Uuid::from_u128(2),
            at(2)
        ));
        assert!(!registry.authorizes(
            Uuid::from_u128(1),
            "participant-1",
            ConsentScope::Deliver,
            Uuid::from_u128(2),
            at(2)
        ));
        assert!(!registry.authorizes(
            Uuid::from_u128(1),
            "participant-1",
            ConsentScope::ProcessRemote,
            Uuid::from_u128(3),
            at(2)
        ));
        assert!(!registry.authorizes(
            Uuid::from_u128(1),
            "participant-1",
            ConsentScope::ProcessRemote,
            Uuid::from_u128(2),
            at(5)
        ));
    }

    #[test]
    fn effective_revocation_denies_an_earlier_matching_grant() {
        let grants = [grant(ConsentScope::ProcessRemote, None)];
        let revocations = [ConsentRevocation {
            contract_version: LOGICAL_CONTRACT_VERSION.into(),
            record_id: Uuid::from_u128(3),
            created_at: at(3),
            grant_id: Uuid::from_u128(1),
            revoked_by: "participant-1".into(),
            revoked_at: at(3),
            reason_code: "USER_REVOKED".into(),
        }];
        let registry = ConsentRegistry::new(&grants, &revocations);
        assert!(registry.authorizes(
            Uuid::from_u128(1),
            "participant-1",
            ConsentScope::ProcessRemote,
            Uuid::from_u128(2),
            at(2)
        ));
        assert!(!registry.authorizes(
            Uuid::from_u128(1),
            "participant-1",
            ConsentScope::ProcessRemote,
            Uuid::from_u128(2),
            at(4)
        ));
    }
}
