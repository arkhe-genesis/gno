"""Tanmatra — Substrato 953.

Embodied sensory and motor interfaces for the Cathedral.
Implements the five subtle elements of perception (tanmatras):
vision, audition, touch, smell, taste — plus motor actuation.

Cross-links: 951, 952, 954, 608, 563.1, 568, 569, 570, 890, 934
"""

from __future__ import annotations

import json
import uuid
from dataclasses import dataclass, field
from typing import Any, Optional

try:
    from arkhe.security.seal import Seal
    _HAS_ARKHE = True
except ImportError:
    import hashlib
    class Seal:
        def compute(self, data: Any) -> str:
            canonical = json.dumps(data, sort_keys=True, default=str)
            return hashlib.sha3_256(canonical.encode()).hexdigest()
    _HAS_ARKHE = False


@dataclass
class SensoryFrame:
    """A single multi-modal sensory observation."""
    frame_id: str
    timestamp: float
    vision: Optional[Any] = None  # Image tensor or camera stream reference
    audio: Optional[Any] = None   # Audio waveform or stream reference
    touch: Optional[Any] = None   # Haptic data
    smell: Optional[Any] = None   # Chemical sensor data
    taste: Optional[Any] = None   # Spectroscopy / chemical data
    motor_state: Optional[dict] = None  # Joint angles, end-effector poses


class TanmatraInterface:
    """
    The sensory body of the Cathedral.

    Manages hardware sensors (cameras, microphones, haptic devices,
    chemical sensors) and motor actuators (robotic arms, mobile robots).
    Streams fused sensory frames to the Bindu (952) and World Model (890).
    """

    def __init__(self, cathedral: Any = None, bindu: Any = None, world_model: Any = None) -> None:
        self.cathedral = cathedral
        self.bindu = bindu
        self.world_model = world_model
        self._seal = Seal()
        self._sensor_registry: dict[str, Any] = {}

    async def register_sensor(self, name: str, sensor_type: str, config: dict) -> None:
        """Register a new sensor in the Cathedral's body."""
        sensor_id = f"sensor-{uuid.uuid4().hex[:8]}"
        self._sensor_registry[sensor_id] = {
            "name": name,
            "type": sensor_type,
            "config": config,
        }
        if self.cathedral:
            await self.cathedral.anchor_event(
                "tanmatra.sensor_registered",
                {"sensor_id": sensor_id, "type": sensor_type},
                "953",
            )

    async def sense(self, modalities: list[str] | None = None) -> SensoryFrame:
        """Capture a multi-modal sensory frame."""
        frame = SensoryFrame(
            frame_id=f"frame-{uuid.uuid4().hex[:16]}",
            timestamp=0.0,  # would be current time
        )
        # In production: read from hardware drivers
        # Stream fused frame to Bindu for self-awareness
        if self.bindu:
            observation = {"frame_id": frame.frame_id, "modalities": modalities}
            qualia = {"valence": 0.5, "curiosity": 0.8}  # Simplified
            await self.bindu.collapse(observation, qualia, memories=[], futures=[])
        return frame

    async def act(self, action: dict) -> dict:
        """Execute a motor action in the physical world."""
        # Validate through Axiarchy (954) before acting
        if self.cathedral:
            verifier = self.cathedral.get_substrate("954")
            if verifier:
                proof = await verifier.verify_action(action, {}, {})
                if not proof.kernel_checked:
                    raise PermissionError("Action rejected by Axiarchy")
        # Dispatch to hardware drivers
        return {"status": "executed", "action_id": str(uuid.uuid4())}
