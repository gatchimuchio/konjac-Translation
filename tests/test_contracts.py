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
    record("ConsentGrant", "10000000-0000-4000-8000-000000000001", principal_id="owner", scope="process_local", subject_ids=[], granted_at=STAMP),
    record("SourceAsset", "10000000-0000-4000-8000-000000000002", media_kind="text", origin="user_input", media_type="text/plain", retention="ephemeral"),
    record("TranslationSession", "10000000-0000-4000-8000-000000000003", revision=1, status="active", source_language="ja", target_languages=["en"], participants=[{"participant_id": "10000000-0000-4000-8000-000000000004", "role": "owner", "consent_grant_ids": []}], context_refs=[], terminology_refs=[], processing_location="local_only", consent_grant_ids=[], retention="ephemeral"),
    record("SourceSpan", "10000000-0000-4000-8000-000000000005", asset_id="10000000-0000-4000-8000-000000000002", session_id="10000000-0000-4000-8000-000000000003", revision=1, source_text="こんにちは", source_language="ja", observation_origin="typed"),
    record("TranslationRequest", "10000000-0000-4000-8000-000000000006", session_id="10000000-0000-4000-8000-000000000003", session_revision=1, revision=1, source_span_ids=["10000000-0000-4000-8000-000000000005"], input_revision=1, target_languages=["en"], context_refs=[], terminology_refs=[], requested_outputs=["text"], execution_mode="batch", processing_location="local_only", idempotency_key="konjac-request-0001", attempt=1),
    record("EngineExecution", "10000000-0000-4000-8000-000000000007", request_id="10000000-0000-4000-8000-000000000006", engine_name="MINIDORA-BETA", engine_version="0.6.0b1", engine_revision="fe5822de63644bdaea9208a98046f71684dffae2", public_entry_point="minidora.engine_beta.ミニドラβ実行機", capability_id="translation.text", outcome="UNSUPPORTED", started_at=STAMP, finished_at=STAMP, diagnostic_code="CAPABILITY_NOT_EXPOSED"),
    record("TranslationVariant", "10000000-0000-4000-8000-000000000008", request_id="10000000-0000-4000-8000-000000000006", source_span_id="10000000-0000-4000-8000-000000000005", target_language="en", revision=1, status="UNSUPPORTED", text=None, produced_at=STAMP),
    record("TranslationResult", "10000000-0000-4000-8000-000000000009", request_id="10000000-0000-4000-8000-000000000006", request_revision=1, status="UNSUPPORTED", variant_ids=[], engine_execution_ids=["10000000-0000-4000-8000-000000000007"], failure_code="CAPABILITY_NOT_EXPOSED", completed_at=STAMP),
    record("DeliveryArtifact", "10000000-0000-4000-8000-000000000010", request_id="10000000-0000-4000-8000-000000000006", variant_id="10000000-0000-4000-8000-000000000008", format="text", destination="local-view", payload_ref={"kind": "local", "key": "result/10000000-0000-4000-8000-000000000009"}, status="READY"),
    record("TranslationEvent", "10000000-0000-4000-8000-000000000011", request_id="10000000-0000-4000-8000-000000000006", sequence=1, event_type="unsupported", occurred_at=STAMP),
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

    def test_translation_request_binds_session_revision_and_execution_mode(self) -> None:
        request = deepcopy(SAMPLES[4])
        for field in ("session_revision", "revision", "execution_mode"):
            incomplete = deepcopy(request)
            incomplete.pop(field)
            with self.subTest(missing=field):
                self.assertFalse(VALIDATOR.is_valid(incomplete))

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
        result.pop("failure_code")
        self.assertTrue(VALIDATOR.is_valid(result))

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
