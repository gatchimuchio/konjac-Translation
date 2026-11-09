"""Checks that capability lookup fails closed at the dependency boundary."""

import unittest
from importlib import metadata
from unittest.mock import patch

from konjac_translation.engine_profile import CapabilityState, inspect_capability


class MinidoraCapabilityProfileTests(unittest.TestCase):
    def test_audited_beta_reports_translation_as_unsupported(self) -> None:
        for capability in (
            "translation.text",
            "translation.multilingual",
            "translation.streaming",
        ):
            with self.subTest(capability=capability):
                decision = inspect_capability(capability, "0.6.0b1")
                self.assertEqual(decision.state, CapabilityState.UNSUPPORTED)
                self.assertEqual(decision.engine_version, "0.6.0b1")
                self.assertEqual(
                    decision.source_revision,
                    "fe5822de63644bdaea9208a98046f71684dffae2",
                )

    def test_unlisted_capability_is_unverified_not_assumed_supported(self) -> None:
        decision = inspect_capability("translation.future", "0.6.0b1")
        self.assertEqual(decision.state, CapabilityState.UNVERIFIED)
        self.assertEqual(decision.reason_code, "CAPABILITY_NOT_AUDITED")

    def test_uninspected_version_is_unverified(self) -> None:
        decision = inspect_capability("translation.text", "0.7.0")
        self.assertEqual(decision.state, CapabilityState.UNVERIFIED)
        self.assertEqual(decision.reason_code, "MINIDORA_VERSION_NOT_AUDITED")

    def test_missing_engine_is_unverified(self) -> None:
        with patch(
            "konjac_translation.engine_profile.metadata.version",
            side_effect=metadata.PackageNotFoundError("minidora-notnn"),
        ):
            decision = inspect_capability("translation.text", None)
        self.assertIn(
            decision.reason_code,
            {"MINIDORA_NOT_INSTALLED"},
        )


if __name__ == "__main__":
    unittest.main()
