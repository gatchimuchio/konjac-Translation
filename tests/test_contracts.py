"""Acceptance checks for the versioned, platform-neutral JSON contract."""

import json
import unittest
from copy import deepcopy
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker
from openapi_spec_validator import validate as validate_openapi


ROOT = Path(__file__).resolve().parents[1]
SCHEMA = json.loads((ROOT / "contracts/konjac-v1.schema.json").read_text(encoding="utf-8"))
VALIDATOR = Draft202012Validator(SCHEMA, format_checker=FormatChecker())
OPENAPI_PATH = ROOT / "contracts/openapi-v1.json"
STAMP = "2026-11-09T00:00:00Z"


def record(record_type: str, record_id: str, **fields: object) -> dict[str, object]:
    return {
        "schema_version": "1.0.0",
        "record_id": record_id,
        "record_type": record_type,
        "created_at": STAMP,
        **fields,
    }


SAMPLES = [
    record("ConsentGrant", "10000000-0000-4000-8000-000000000001", principal_id="owner", scope="process_local", subject_ids=["10000000-0000-4000-8000-000000000002"], granted_at=STAMP, expires_at=None, evidence_ref=None),
    record("SourceAsset", "10000000-0000-4000-8000-000000000002", media_kind="text", origin="user_input", media_type="text/plain", retention="ephemeral", metadata={}),
    record("TranslationSession", "10000000-0000-4000-8000-000000000003", revision=1, status="active", source_language=None, target_languages=["en"], participants=[{"participant_id": "10000000-0000-4000-8000-000000000004", "role": "owner", "preferred_language": None, "consent_grant_ids": []}], context_refs=[], terminology_refs=[], processing_location="local_only", consent_grant_ids=[], retention="ephemeral"),
    record("SourceSpan", "10000000-0000-4000-8000-000000000005", asset_id="10000000-0000-4000-8000-000000000002", session_id="10000000-0000-4000-8000-000000000003", revision=1, payload={"kind": "text", "text": "こんにちは"}, source_language="ja", observation_origin="typed"),
    record("TranslationRequest", "10000000-0000-4000-8000-000000000006", session_id="10000000-0000-4000-8000-000000000003", session_revision=1, revision=1, source_span_ids=["10000000-0000-4000-8000-000000000005"], input_revision=1, target_languages=["en"], context_refs=[], terminology_refs=[], requested_outputs=["text"], execution_mode="batch", processing_location="local_only", processing_consent_id=None, idempotency_key="konjac-request-0001", attempt=1, deadline=None, cancel_requested_at=None),
    record("EngineExecution", "10000000-0000-4000-8000-000000000007", request_id="10000000-0000-4000-8000-000000000006", engine_name="MINIDORA-BETA", engine_version="0.6.0b1", engine_revision="fe5822de63644bdaea9208a98046f71684dffae2", public_entry_point="minidora.engine_beta.ミニドラβ実行機", capability={"capability_id": "translation.text", "state": "UNSUPPORTED", "engine_version": "0.6.0b1", "engine_revision": "fe5822de63644bdaea9208a98046f71684dffae2", "reason_code": "NO_PUBLIC_TRANSLATION_OPERATION", "observed_at": STAMP}, input_fingerprint="0" * 64, response_fingerprint=None, outcome="UNSUPPORTED", started_at=STAMP, finished_at=STAMP, diagnostic_code="NO_PUBLIC_TRANSLATION_OPERATION", trace_ref=None),
    record("TranslationVariant", "10000000-0000-4000-8000-000000000008", request_id="10000000-0000-4000-8000-000000000006", source_span_id="10000000-0000-4000-8000-000000000005", target_language="en", revision=1, status="UNSUPPORTED", text=None, engine_execution_id=None, provenance_ref=None, speaker_id=None, position=None, produced_at=STAMP),
    record("TranslationResult", "10000000-0000-4000-8000-000000000009", request_id="10000000-0000-4000-8000-000000000006", request_revision=1, status="UNSUPPORTED", variant_ids=[], engine_execution_ids=["10000000-0000-4000-8000-000000000007"], failure_code="CAPABILITY_NOT_EXPOSED", completed_at=STAMP),
    record("DeliveryArtifact", "10000000-0000-4000-8000-000000000010", variant_id="10000000-0000-4000-8000-000000000008", format="text", payload_ref={"kind": "local", "key": "result/10000000-0000-4000-8000-000000000009"}, destination={"kind": "local", "key": "local-view"}, position=None, delivery_consent_id=None, status="PREPARED", delivered_at=None, audit_id="10000000-0000-4000-8000-000000000014"),
    record("TranslationEvent", "10000000-0000-4000-8000-000000000011", request_id="10000000-0000-4000-8000-000000000006", sequence=1, event_type="unsupported", variant_id=None, result_id="10000000-0000-4000-8000-000000000009", delivery_artifact_id=None, occurred_at=STAMP),
    record("ConsentRevocation", "10000000-0000-4000-8000-000000000012", grant_id="10000000-0000-4000-8000-000000000001", revoked_by="owner", revoked_at=STAMP, reason_code="USER_REQUEST"),
]


class ContractSchemaTests(unittest.TestCase):
    def test_schema_is_valid_draft_2020_12(self) -> None:
        Draft202012Validator.check_schema(SCHEMA)

    def test_openapi_is_valid_and_resolves_the_shared_schemas(self) -> None:
        spec = json.loads(OPENAPI_PATH.read_text(encoding="utf-8"))
        validate_openapi(spec, base_uri=OPENAPI_PATH.as_uri())

    def test_each_record_family_has_a_valid_example(self) -> None:
        for sample in SAMPLES:
            with self.subTest(record_type=sample["record_type"]):
                VALIDATOR.validate(sample)

    def test_events_require_references_for_their_event_type(self) -> None:
        event = deepcopy(SAMPLES[9])
        for reference in ("variant_id", "result_id", "delivery_artifact_id"):
            event.pop(reference, None)

        required_references = {
            "variant_updated": "variant_id",
            "completed": "result_id",
            "unsupported": "result_id",
            "suspended": "result_id",
            "failed": "result_id",
            "cancelled": "result_id",
            "timed_out": "result_id",
            "delivery_updated": "delivery_artifact_id",
        }
        for event_type, reference in required_references.items():
            with self.subTest(event_type=event_type):
                event["event_type"] = event_type
                event[reference] = None
                self.assertFalse(VALIDATOR.is_valid(event))
                event[reference] = "10000000-0000-4000-8000-000000000013"
                VALIDATOR.validate(event)
                event.pop(reference)
                self.assertFalse(VALIDATOR.is_valid(event))

        event = deepcopy(SAMPLES[9])
        event["sequence"] = 18446744073709551615
        VALIDATOR.validate(event)
        event["sequence"] += 1
        self.assertFalse(VALIDATOR.is_valid(event))

    def test_source_asset_projection_matches_logical_references(self) -> None:
        asset = deepcopy(SAMPLES[1])
        asset.update(
            {
                "content_ref": {
                    "kind": "local",
                    "key": "asset/source-1",
                    "media_type": "image/png",
                    "sha256": None,
                    "size_bytes": 128,
                },
                "origin": "camera",
                "original_fingerprint": "a" * 64,
                "owner_ref": {"kind": "local", "key": "owner/1"},
                "rights_ref": None,
                "capture_consent_id": "10000000-0000-4000-8000-000000000014",
                "time_basis": "unix_ms_from_capture_start",
            }
        )
        VALIDATOR.validate(asset)

        asset["capture_consent_id"] = None
        self.assertFalse(VALIDATOR.is_valid(asset))
        asset["capture_consent_id"] = "10000000-0000-4000-8000-000000000014"
        asset["content_ref"] = None
        self.assertTrue(VALIDATOR.is_valid(asset))
        for unsupported_field in ("content_digest", "source_uri", "attribution"):
            incompatible = deepcopy(asset)
            incompatible[unsupported_field] = "not in the logical SourceAsset"
            with self.subTest(field=unsupported_field):
                self.assertFalse(VALIDATOR.is_valid(incompatible))

        asset["metadata"] = {"nested": {"raw": "not scalar metadata"}}
        self.assertFalse(VALIDATOR.is_valid(asset))

        asset["metadata"] = {}
        asset["original_fingerprint"] = "A" * 64
        self.assertFalse(VALIDATOR.is_valid(asset))

        oversized_reference = deepcopy(SAMPLES[1])
        oversized_reference["content_ref"] = {
            "kind": "local",
            "key": "asset/source-1",
            "size_bytes": 18446744073709551616,
        }
        self.assertFalse(VALIDATOR.is_valid(oversized_reference))

    def test_source_span_uses_tagged_payload_and_position_records(self) -> None:
        span = deepcopy(SAMPLES[3])
        span.update(
            {
                "source_language": None,
                "confidence": None,
                "speaker_id": None,
                "position": None,
                "provenance_ref": None,
            }
        )
        VALIDATOR.validate(span)

        positions = [
            {"kind": "audio_time", "start_ms": 0, "end_ms": 100},
            {"kind": "video_time", "start_ms": 1, "end_ms": 2},
            {"kind": "document_anchor", "anchor": "paragraph/1", "page": None},
            {"kind": "image_bounds", "page": 1, "x": 0.1, "y": 0.2, "width": 0.3, "height": 0.4},
        ]
        for position in positions:
            with self.subTest(position=position["kind"]):
                span["position"] = position
                VALIDATOR.validate(span)

        span["payload"] = {
            "kind": "media",
            "reference": {"kind": "remote", "key": "media/audio-1", "media_type": None, "sha256": None, "size_bytes": None},
        }
        VALIDATOR.validate(span)
        span["payload"] = {"kind": "text", "text": ""}
        self.assertFalse(VALIDATOR.is_valid(span))
        span["payload"] = {"kind": "text", "text": "ok", "extra": "rejected"}
        self.assertFalse(VALIDATOR.is_valid(span))

    def test_engine_execution_preserves_capability_and_fingerprint_evidence(self) -> None:
        execution = deepcopy(SAMPLES[5])
        VALIDATOR.validate(execution)

        execution["capability"]["state"] = "SUPPORTED"
        self.assertFalse(VALIDATOR.is_valid(execution))

        execution = deepcopy(SAMPLES[5])
        execution["outcome"] = "COMPLETED"
        execution["capability"]["state"] = "SUPPORTED"
        execution["response_fingerprint"] = "b" * 64
        VALIDATOR.validate(execution)

        execution["response_fingerprint"] = None
        self.assertFalse(VALIDATOR.is_valid(execution))
        execution = deepcopy(SAMPLES[5])
        execution["input_fingerprint"] = "invalid"
        self.assertFalse(VALIDATOR.is_valid(execution))

    def test_translation_variant_distinguishes_provisional_and_final_evidence(self) -> None:
        variant = deepcopy(SAMPLES[6])
        VALIDATOR.validate(variant)

        variant["status"] = "PROVISIONAL"
        VALIDATOR.validate(variant)
        variant["text"] = "interim text"
        self.assertFalse(VALIDATOR.is_valid(variant))
        variant["engine_execution_id"] = "10000000-0000-4000-8000-000000000007"
        VALIDATOR.validate(variant)

        variant["status"] = "FINAL"
        variant["text"] = "final text"
        VALIDATOR.validate(variant)
        variant["engine_execution_id"] = None
        self.assertFalse(VALIDATOR.is_valid(variant))

    def test_delivery_artifact_matches_logical_states_and_references(self) -> None:
        artifact = deepcopy(SAMPLES[8])
        VALIDATOR.validate(artifact)

        artifact["status"] = "AUTHORIZED"
        self.assertFalse(VALIDATOR.is_valid(artifact))
        artifact["delivery_consent_id"] = "10000000-0000-4000-8000-000000000001"
        VALIDATOR.validate(artifact)

        artifact["status"] = "DELIVERED"
        self.assertFalse(VALIDATOR.is_valid(artifact))
        artifact["delivered_at"] = STAMP
        VALIDATOR.validate(artifact)

        artifact["status"] = "DELIVERY_UNKNOWN"
        artifact["delivered_at"] = None
        VALIDATOR.validate(artifact)
        artifact["delivery_consent_id"] = None
        self.assertFalse(VALIDATOR.is_valid(artifact))

        artifact["status"] = "DELIVERY_FAILED"
        self.assertTrue(VALIDATOR.is_valid(artifact))
        artifact["delivered_at"] = STAMP
        self.assertFalse(VALIDATOR.is_valid(artifact))

        artifact["status"] = "PREPARED"
        artifact["delivered_at"] = None
        artifact["destination"] = "local-view"
        self.assertFalse(VALIDATOR.is_valid(artifact))
        artifact["destination"] = {"kind": "local", "key": "local-view"}
        artifact["payload_ref"] = "opaque"
        self.assertFalse(VALIDATOR.is_valid(artifact))
        artifact["payload_ref"] = {"kind": "local", "key": "result/10000000-0000-4000-8000-000000000009"}
        artifact["status"] = "READY"
        self.assertFalse(VALIDATOR.is_valid(artifact))
        artifact["status"] = "PREPARED"
        artifact.pop("audit_id")
        self.assertFalse(VALIDATOR.is_valid(artifact))

    def test_language_tags_follow_bcp47_subtag_order(self) -> None:
        session = deepcopy(SAMPLES[2])
        for tag in [
            "zh-Hant-TW",
            "zh-cmn-Hans-CN",
            "de-CH-1901",
            "en-u-ca-gregory",
            "x-private",
            "i-klingon",
            "en-GB-oed",
        ]:
            session["target_languages"] = [tag]
            self.assertTrue(VALIDATOR.is_valid(session), tag)

        for tag in ["en-a", "en-x", "en-US-Latn", "a-bbb", "en--US", "x"]:
            session["target_languages"] = [tag]
            self.assertFalse(VALIDATOR.is_valid(session), tag)

    def test_remote_processing_requires_explicit_consent_reference(self) -> None:
        request = deepcopy(SAMPLES[4])
        request["processing_location"] = "remote_with_consent"
        self.assertFalse(VALIDATOR.is_valid(request))
        request["processing_consent_id"] = "10000000-0000-4000-8000-000000000001"
        self.assertTrue(VALIDATOR.is_valid(request))

    def test_consent_grant_projection_matches_nullable_options_and_scope(self) -> None:
        grant = deepcopy(SAMPLES[0])
        VALIDATOR.validate(grant)

        for field in ("expires_at", "evidence_ref"):
            incomplete = deepcopy(grant)
            incomplete.pop(field)
            with self.subTest(missing=field):
                self.assertFalse(VALIDATOR.is_valid(incomplete))

        grant["subject_ids"] = []
        self.assertFalse(VALIDATOR.is_valid(grant))
        grant["subject_ids"] = ["10000000-0000-4000-8000-000000000002"] * 2
        self.assertFalse(VALIDATOR.is_valid(grant))

    def test_session_projection_matches_nullable_options_and_revision_bounds(self) -> None:
        session = deepcopy(SAMPLES[2])
        VALIDATOR.validate(session)

        session["source_language"] = "und"
        VALIDATOR.validate(session)
        session["source_language"] = None
        session["revision"] = 18446744073709551615
        VALIDATOR.validate(session)
        session["revision"] += 1
        self.assertFalse(VALIDATOR.is_valid(session))

        session = deepcopy(SAMPLES[2])
        session.pop("source_language")
        self.assertFalse(VALIDATOR.is_valid(session))
        session = deepcopy(SAMPLES[2])
        session["participants"][0].pop("preferred_language")
        self.assertFalse(VALIDATOR.is_valid(session))

        duplicate_ref = {"kind": "local", "key": "context/1"}
        session = deepcopy(SAMPLES[2])
        session["context_refs"] = [duplicate_ref, duplicate_ref]
        self.assertFalse(VALIDATOR.is_valid(session))
        session = deepcopy(SAMPLES[2])
        session["terminology_refs"] = [duplicate_ref, duplicate_ref]
        self.assertFalse(VALIDATOR.is_valid(session))

        session = deepcopy(SAMPLES[2])
        session["processing_location"] = "remote_with_consent"
        self.assertFalse(VALIDATOR.is_valid(session))
        session["consent_grant_ids"] = ["10000000-0000-4000-8000-000000000001"]
        self.assertTrue(VALIDATOR.is_valid(session))

    def test_translation_request_binds_session_revision_and_execution_mode(self) -> None:
        request = deepcopy(SAMPLES[4])
        for field in ("session_revision", "revision", "execution_mode", "processing_consent_id", "deadline", "cancel_requested_at"):
            incomplete = deepcopy(request)
            incomplete.pop(field)
            with self.subTest(missing=field):
                self.assertFalse(VALIDATOR.is_valid(incomplete))

        for field in ("session_revision", "revision", "input_revision"):
            request[field] = 18446744073709551615
        request["attempt"] = 4294967295
        request["idempotency_key"] = "x" * 256
        VALIDATOR.validate(request)

        request["session_revision"] += 1
        self.assertFalse(VALIDATOR.is_valid(request))
        request["session_revision"] = 1
        request["attempt"] += 1
        self.assertFalse(VALIDATOR.is_valid(request))
        request["attempt"] = 1
        request["idempotency_key"] = "x" * 257
        self.assertFalse(VALIDATOR.is_valid(request))
        request["idempotency_key"] = "界" * 15
        self.assertFalse(VALIDATOR.is_valid(request))
        request["idempotency_key"] = "界" * 16
        VALIDATOR.validate(request)

        reference = {"kind": "local", "key": "context/1"}
        request["context_refs"] = [reference, reference]
        self.assertFalse(VALIDATOR.is_valid(request))

    def test_final_variant_requires_engine_evidence_and_nonempty_text(self) -> None:
        variant = deepcopy(SAMPLES[6])
        variant["status"] = "FINAL"
        variant["text"] = "Hello"
        self.assertFalse(VALIDATOR.is_valid(variant))
        variant["engine_execution_id"] = "10000000-0000-4000-8000-000000000007"
        self.assertTrue(VALIDATOR.is_valid(variant))
        variant["text"] = ""
        self.assertFalse(VALIDATOR.is_valid(variant))

    def test_completed_result_requires_variants_and_engine_runs(self) -> None:
        result = deepcopy(SAMPLES[7])
        result["status"] = "COMPLETED"
        result["completed_at"] = STAMP
        result["failure_code"] = None
        self.assertFalse(VALIDATOR.is_valid(result))
        result["variant_ids"] = ["10000000-0000-4000-8000-000000000008"]
        result["engine_execution_ids"] = ["10000000-0000-4000-8000-000000000007"]
        self.assertTrue(VALIDATOR.is_valid(result))

    def test_translation_result_status_conditions_match_rust_option_fields(self) -> None:
        result = deepcopy(SAMPLES[7])
        result["status"] = "PENDING"
        result["variant_ids"] = []
        result["engine_execution_ids"] = []
        result["failure_code"] = None
        result["completed_at"] = None
        VALIDATOR.validate(result)

        result.pop("completed_at")
        self.assertFalse(VALIDATOR.is_valid(result))
        result["completed_at"] = STAMP
        self.assertFalse(VALIDATOR.is_valid(result))
        result["completed_at"] = None
        result["status"] = "PROCESSING"
        VALIDATOR.validate(result)

        result["status"] = "UNSUPPORTED"
        result["completed_at"] = STAMP
        self.assertFalse(VALIDATOR.is_valid(result))
        result["failure_code"] = "CAPABILITY_NOT_EXPOSED"
        VALIDATOR.validate(result)
        result["failure_code"] = ""
        self.assertFalse(VALIDATOR.is_valid(result))
        result["failure_code"] = "x" * 129
        self.assertFalse(VALIDATOR.is_valid(result))

        result["status"] = "PENDING"
        result["failure_code"] = None
        result["completed_at"] = None
        result["request_revision"] = 18446744073709551615
        VALIDATOR.validate(result)
        result["request_revision"] += 1
        self.assertFalse(VALIDATOR.is_valid(result))

    def test_unsupported_result_cannot_include_translation_variants(self) -> None:
        result = deepcopy(SAMPLES[7])
        result["variant_ids"] = ["10000000-0000-4000-8000-000000000008"]
        self.assertFalse(VALIDATOR.is_valid(result))

    def test_external_delivery_requires_a_delivery_grant(self) -> None:
        artifact = deepcopy(SAMPLES[8])
        artifact["status"] = "DELIVERED"
        artifact["delivered_at"] = STAMP
        self.assertFalse(VALIDATOR.is_valid(artifact))
        artifact["delivery_consent_id"] = "10000000-0000-4000-8000-000000000001"
        self.assertTrue(VALIDATOR.is_valid(artifact))

    def test_unsupported_variant_cannot_carry_fabricated_translation(self) -> None:
        variant = deepcopy(SAMPLES[6])
        variant["text"] = "Hello"
        self.assertFalse(VALIDATOR.is_valid(variant))

    def test_unknown_fields_are_rejected(self) -> None:
        source = deepcopy(SAMPLES[1])
        source["internal_reasoning"] = "must not cross the public contract"
        self.assertFalse(VALIDATOR.is_valid(source))


if __name__ == "__main__":
    unittest.main()
