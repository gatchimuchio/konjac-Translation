"""Resolve MINIDORA capabilities from audited, exact distribution profiles.

This module is a capability gate. It does not execute a translation engine and
never substitutes another provider when the installed engine lacks a route.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from enum import StrEnum
from importlib import metadata, resources
from typing import Any


class CapabilityState(StrEnum):
    SUPPORTED = "SUPPORTED"
    UNSUPPORTED = "UNSUPPORTED"
    UNVERIFIED = "UNVERIFIED"


@dataclass(frozen=True)
class CapabilityDecision:
    capability_id: str
    state: CapabilityState
    reason_code: str
    engine_name: str | None
    engine_version: str | None
    source_revision: str | None


@dataclass(frozen=True)
class EngineProfile:
    engine: str
    distribution: str
    version: str
    frozen_source_baseline: str
    observed_repository_head: str
    capabilities: dict[str, dict[str, str]]

    @classmethod
    def load(cls, version: str) -> EngineProfile | None:
        """Return the profile only for a version whose public API was audited."""
        if version != "0.6.0b1":
            return None
        resource = resources.files("konjac_translation").joinpath(
            "data/minidora_beta_0_6_0b1.json"
        )
        data: dict[str, Any] = json.loads(resource.read_text(encoding="utf-8"))
        return cls(
            engine=data["engine"],
            distribution=data["distribution"],
            version=data["version"],
            frozen_source_baseline=data["frozen_source_baseline"],
            observed_repository_head=data["observed_repository_head"],
            capabilities=data["capabilities"],
        )

    def decide(self, capability_id: str) -> CapabilityDecision:
        capability = self.capabilities.get(capability_id)
        if capability is None:
            return CapabilityDecision(
                capability_id=capability_id,
                state=CapabilityState.UNVERIFIED,
                reason_code="CAPABILITY_NOT_AUDITED",
                engine_name=self.engine,
                engine_version=self.version,
                source_revision=self.frozen_source_baseline,
            )
        return CapabilityDecision(
            capability_id=capability_id,
            state=CapabilityState(capability["state"]),
            reason_code=capability["reason_code"],
            engine_name=self.engine,
            engine_version=self.version,
            source_revision=self.frozen_source_baseline,
        )


def inspect_capability(
    capability_id: str,
    installed_version: str | None = None,
) -> CapabilityDecision:
    """Report a known capability or preserve uncertainty for unknown versions."""
    if installed_version is None:
        try:
            installed_version = metadata.version("minidora-notnn")
        except metadata.PackageNotFoundError:
            return CapabilityDecision(
                capability_id,
                CapabilityState.UNVERIFIED,
                "MINIDORA_NOT_INSTALLED",
                "MINIDORA-BETA",
                None,
                None,
            )

    profile = EngineProfile.load(installed_version)
    if profile is None:
        return CapabilityDecision(
            capability_id,
            CapabilityState.UNVERIFIED,
            "MINIDORA_VERSION_NOT_AUDITED",
            "MINIDORA-BETA",
            installed_version,
            None,
        )
    return profile.decide(capability_id)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("capability", nargs="?", default="translation.text")
    args = parser.parse_args()
    decision = inspect_capability(args.capability)
    print(json.dumps(asdict(decision), ensure_ascii=False, indent=2))
    return 0 if decision.state is CapabilityState.SUPPORTED else 2


if __name__ == "__main__":
    sys.exit(main())
