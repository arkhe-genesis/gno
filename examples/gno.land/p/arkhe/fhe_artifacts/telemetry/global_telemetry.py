#!/usr/bin/env python3
"""
ARKHE Global Mesh — Global Telemetry
Substrato 972 — ARKHE-GLOBAL-MESH

Observabilidade distribuida:
- Prometheus metrics
- Distributed tracing (OpenTelemetry)
- Theosis monitoring
- Anomaly detection (Glasswing 944)
"""

import numpy as np
from typing import Dict, List, Optional
from dataclasses import dataclass, field
from datetime import datetime, timezone

@dataclass
class TelemetryPoint:
    node_id: str
    metric: str
    value: float
    timestamp: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    labels: Dict[str, str] = field(default_factory=dict)

class GlobalTelemetry:
    """Telemetria global da malha."""

    def __init__(self, retention_hours: int = 24):
        self.retention_hours = retention_hours
        self.metrics: List[TelemetryPoint] = []
        self.alerts: List[Dict] = []

    def record(self, point: TelemetryPoint):
        """Registra metrica."""
        self.metrics.append(point)

    def query(self, node_id: Optional[str] = None, metric: Optional[str] = None) -> List[TelemetryPoint]:
        """Consulta metricas."""
        results = self.metrics

        if node_id:
            results = [m for m in results if m.node_id == node_id]
        if metric:
            results = [m for m in results if m.metric == metric]

        return results

    def detect_anomalies(self, threshold: float = 3.0) -> List[Dict]:
        """Detecta anomalias usando desvio padrao."""
        anomalies = []

        # Agrupar por metrica
        by_metric: Dict[str, List[float]] = {}
        for m in self.metrics:
            if m.metric not in by_metric:
                by_metric[m.metric] = []
            by_metric[m.metric].append(m.value)

        for metric, values in by_metric.items():
            if len(values) < 10:
                continue

            mean = np.mean(values)
            std = np.std(values)

            for m in self.metrics:
                if m.metric == metric:
                    z_score = abs(m.value - mean) / std if std > 0 else 0
                    if z_score > threshold:
                        anomalies.append({
                            "node_id": m.node_id,
                            "metric": metric,
                            "value": m.value,
                            "z_score": z_score,
                            "timestamp": m.timestamp,
                        })

        return anomalies
