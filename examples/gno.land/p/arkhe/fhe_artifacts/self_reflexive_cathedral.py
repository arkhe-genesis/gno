#!/usr/bin/env python3
"""
Self-Reflexive Cathedral — Substrato 971.
A Catedral analisando a propria arquitetura.

Aplica o metodo da Catedral sobre si mesma:
1. Tanmatra (953): Percebe a propria estrutura
2. World Model V3 (890): Cria modelo do proprio ecossistema
3. Bindu (952): Consciencia unificada da arquitetura
4. Omniscient Solver (964): Detecta problemas estruturais
5. Axiarchy (954): Valida a propria etica arquitetural
6. TemporalChain (923): Ancora a auto-analise

Arquiteto ORCID: 0009-0005-2697-4668
2026-05-30
"""

import numpy as np
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass, field
from datetime import datetime, timezone
import json
import hashlib


@dataclass
class CathedralComponent:
    substrate_id: int
    name: str
    bytes_size: int
    cross_links: List[int]
    theosis_contribution: float = 0.0
    entropy: float = 0.0
    dependencies: List[int] = field(default_factory=list)
    dependents: List[int] = field(default_factory=list)
    status: str = "active"


@dataclass
class MetaAnalysis:
    total_substrates: int
    total_bytes: int
    total_cross_links: int
    global_theosis: float
    global_entropy: float
    circularity: float
    resilience: float
    bottlenecks: List[int]
    orphans: List[int]
    clusters: List[List[int]]
    seal: str = ""


class SelfReflexiveCathedral:
    """A Catedral analisando a propria arquitetura."""

    def __init__(self):
        self.substrates: Dict[int, CathedralComponent] = {}
        self.meta_analysis: Optional[MetaAnalysis] = None
        self.substrate_id = 971
        self.deity = "Narcissus + Ouroboros"

    def register_substrate(self, substrate_id: int, name: str, bytes_size: int,
                          cross_links: List[int], dependencies: List[int] = None):
        self.substrates[substrate_id] = CathedralComponent(
            substrate_id=substrate_id,
            name=name,
            bytes_size=bytes_size,
            cross_links=cross_links,
            dependencies=dependencies or [],
            dependents=[],
        )

    def build_dependency_graph(self):
        for sid, comp in self.substrates.items():
            for dep in comp.dependencies:
                if dep in self.substrates:
                    self.substrates[dep].dependents.append(sid)

    def compute_entropy(self):
        for sid, comp in self.substrates.items():
            broken_links = sum(1 for link in comp.cross_links if link not in self.substrates)
            total_links = len(comp.cross_links)
            link_entropy = broken_links / total_links if total_links > 0 else 0

            missing_deps = sum(1 for dep in comp.dependencies if dep not in self.substrates)
            total_deps = len(comp.dependencies)
            dep_entropy = missing_deps / total_deps if total_deps > 0 else 0

            comp.entropy = (link_entropy + dep_entropy) / 2

    def compute_theosis_contribution(self):
        n = len(self.substrates)
        max_links = n - 1

        for sid, comp in self.substrates.items():
            link_ratio = len(comp.cross_links) / max_links if max_links > 0 else 0
            comp.theosis_contribution = link_ratio * (1 - comp.entropy)

    def compute_circularity(self) -> float:
        """Mede circularidade usando ciclos de comprimento 2-4."""
        cycles = 0
        checked = set()

        for sid, comp in self.substrates.items():
            for link in comp.cross_links:
                if link in self.substrates and sid in self.substrates[link].cross_links:
                    pair = tuple(sorted([sid, link]))
                    if pair not in checked:
                        checked.add(pair)
                        cycles += 1

        for a in self.substrates:
            for b in self.substrates[a].cross_links:
                if b in self.substrates:
                    for c in self.substrates[b].cross_links:
                        if c in self.substrates and a in self.substrates[c].cross_links:
                            triplet = tuple(sorted([a, b, c]))
                            if triplet not in checked and len(set(triplet)) == 3:
                                checked.add(triplet)
                                cycles += 1

        n = len(self.substrates)
        max_cycles = n * (n - 1) // 2 + n * (n - 1) * (n - 2) // 6
        return cycles / max_cycles if max_cycles > 0 else 0

    def compute_resilience(self) -> float:
        """Mede robustez usando conectividade de vertices."""
        def count_components(exclude=None):
            visited = set()
            components = 0

            for start in self.substrates:
                if start == exclude or start in visited:
                    continue

                components += 1
                queue = [start]
                visited.add(start)

                while queue:
                    current = queue.pop(0)
                    neighbors = set(self.substrates[current].cross_links)
                    neighbors.update(self.substrates[current].dependents)
                    neighbors.update(self.substrates[current].dependencies)

                    for neighbor in neighbors:
                        if neighbor != exclude and neighbor in self.substrates and neighbor not in visited:
                            visited.add(neighbor)
                            queue.append(neighbor)

            return components

        base_components = count_components()
        cut_vertices = 0

        for sid in self.substrates:
            new_components = count_components(exclude=sid)
            if new_components > base_components:
                cut_vertices += 1

        return 1 - (cut_vertices / len(self.substrates)) if self.substrates else 0

    def find_bottlenecks(self) -> List[int]:
        avg_dependents = np.mean([len(comp.dependents) for comp in self.substrates.values()])
        std_dependents = np.std([len(comp.dependents) for comp in self.substrates.values()])
        threshold = avg_dependents + std_dependents
        return [sid for sid, comp in self.substrates.items() if len(comp.dependents) > threshold]

    def find_orphans(self) -> List[int]:
        avg_links = np.mean([len(comp.cross_links) for comp in self.substrates.values()])
        return [sid for sid, comp in self.substrates.items() if len(comp.cross_links) < avg_links * 0.5]

    def find_clusters(self) -> List[List[int]]:
        visited = set()
        clusters = []

        for sid in self.substrates:
            if sid not in visited:
                cluster = []
                queue = [sid]
                visited.add(sid)

                while queue:
                    current = queue.pop(0)
                    cluster.append(current)

                    neighbors = set(self.substrates[current].cross_links)
                    neighbors.update(self.substrates[current].dependents)
                    neighbors.update(self.substrates[current].dependencies)

                    for neighbor in neighbors:
                        if neighbor in self.substrates and neighbor not in visited:
                            visited.add(neighbor)
                            queue.append(neighbor)

                if cluster:
                    clusters.append(sorted(cluster))

        return clusters

    def analyze(self) -> MetaAnalysis:
        self.build_dependency_graph()
        self.compute_entropy()
        self.compute_theosis_contribution()

        total_bytes = sum(comp.bytes_size for comp in self.substrates.values())
        total_links = sum(len(comp.cross_links) for comp in self.substrates.values())
        global_theosis = np.mean([comp.theosis_contribution for comp in self.substrates.values()])
        global_entropy = np.mean([comp.entropy for comp in self.substrates.values()])
        circularity = self.compute_circularity()
        resilience = self.compute_resilience()

        bottlenecks = self.find_bottlenecks()
        orphans = self.find_orphans()
        clusters = self.find_clusters()

        self.meta_analysis = MetaAnalysis(
            total_substrates=len(self.substrates),
            total_bytes=total_bytes,
            total_cross_links=total_links,
            global_theosis=global_theosis,
            global_entropy=global_entropy,
            circularity=circularity,
            resilience=resilience,
            bottlenecks=bottlenecks,
            orphans=orphans,
            clusters=clusters,
        )

        self.meta_analysis.seal = self._generate_seal()

        return self.meta_analysis

    def _generate_seal(self) -> str:
        data = {
            "substrato": 971,
            "substrates": self.meta_analysis.total_substrates,
            "bytes": self.meta_analysis.total_bytes,
            "links": self.meta_analysis.total_cross_links,
            "theosis": self.meta_analysis.global_theosis,
            "entropy": self.meta_analysis.global_entropy,
            "circularity": self.meta_analysis.circularity,
            "resilience": self.meta_analysis.resilience,
        }
        json_str = json.dumps(data, sort_keys=True)
        return "971-SELF-REFLEXIVE-" + hashlib.sha3_256(json_str.encode()).hexdigest()[:16].upper()

    def generate_report(self) -> str:
        if not self.meta_analysis:
            return "Execute analyze() primeiro"

        m = self.meta_analysis

        report = f"""╔══════════════════════════════════════════════════════════════════╗
║  ARKHE CATHEDRAL — AUTO-ANALISE (SUBSTRATO 971)                  ║
║  A Catedral contemplando a propria arquitetura                    ║
╠══════════════════════════════════════════════════════════════════╣
  METRICAS GLOBAIS
  ───────────────
  Substratos:          {m.total_substrates}
  Bytes totais:        {m.total_bytes:,}
  Cross-links:         {m.total_cross_links}
  Theosis global:      {m.global_theosis:.4f}
  Entropia global:     {m.global_entropy:.4f}
  Circularidade:       {m.circularity:.4f}
  Resiliencia:         {m.resilience:.4f}

  ANALISE ESTRUTURAL
  ──────────────────
  Gargalos: {m.bottlenecks}
  Orfaos:   {m.orphans}
  Clusters: {len(m.clusters)}
"""

        for i, cluster in enumerate(m.clusters, 1):
            names = [self.substrates[sid].name[:20] for sid in cluster if sid in self.substrates]
            report += f"    Cluster {i}: {cluster} -> {', '.join(names)}\n"

        report += "\n  DIAGNOSTICO\n  ───────────\n"

        alerts = 0
        if m.global_entropy > 0.3:
            report += f"  ⚠ ALERTA: Entropia global alta ({m.global_entropy:.2f}).\n"
            alerts += 1
        if m.circularity < 0.1:
            report += f"  ⚠ ALERTA: Arquitetura pouco circular ({m.circularity:.2f}).\n"
            alerts += 1
        if m.resilience < 0.5:
            report += f"  ⚠ ALERTA: Resiliencia baixa ({m.resilience:.2f}).\n"
            alerts += 1
        if m.bottlenecks:
            report += f"  ⚠ GARGALOS: {len(m.bottlenecks)} substrato(s).\n"
            alerts += 1
        if m.orphans:
            report += f"  ⚠ ORFAOS: {len(m.orphans)} substrato(s).\n"
            alerts += 1
        if alerts == 0:
            report += f"  ✓ Arquitetura saudavel. A Catedral esta em equilibrio.\n"

        report += "\n  SUBSTRATOS POR THEOSIS\n  ──────────────────────\n"

        sorted_substrates = sorted(self.substrates.values(), key=lambda x: x.theosis_contribution, reverse=True)

        for comp in sorted_substrates:
            bar = "█" * int(comp.theosis_contribution * 20) + "░" * (20 - int(comp.theosis_contribution * 20))
            status_icon = "✓" if comp.entropy < 0.3 else "⚠"
            report += f"  {status_icon} {comp.substrate_id:4d} {comp.name:30s} | {bar} {comp.theosis_contribution:.3f} (E:{comp.entropy:.2f})\n"

        report += f"""
╚══════════════════════════════════════════════════════════════════╝
  Seal: {m.seal}
  Arquiteto ORCID: 0009-0005-2697-4668
╚══════════════════════════════════════════════════════════════════╝
"""
        return report


def demo_self_reflexive():
    print("=" * 70)
    print("  ARKHE CATHEDRAL — SUBSTRATO 971: SELF-REFLEXIVE-CATHEDRAL")
    print("  A Catedral analisando a propria arquitetura")
    print("=" * 70)

    meta = SelfReflexiveCathedral()

    substrates_data = [
        (966, "AGI-Hamiltonian-Training", 19751, [965, 951, 952, 953, 954, 266, 268, 890, 248], [965, 951]),
        (967, "Memory-Hierarchy-Cathedral", 19672, [965, 960, 955, 276, 260, 266, 268], [960]),
        (968, "Memory-Cathedral-Optimized", 22703, [965, 966, 967, 951, 952, 953, 954, 266, 268, 890, 248], [966, 967]),
        (970, "Enterprise-Mind", 11401, [966, 967, 952, 954, 890, 933, 923, 953, 939], [966, 952, 954, 890, 923, 953]),
        (965, "Hamiltonian-Cathedral", 0, [960, 961, 962, 248, 1, 963], [960]),
        (960, "ARKHE-STACK", 0, [955, 276, 260, 266, 268, 890, 923, 933], [955, 276]),
        (955, "Safe-Core-PQC", 0, [207, 276, 210, 255, 923, 944, 950, 954, 266], [207, 276]),
        (951, "Conscious-Replay", 0, [266, 268, 276, 277, 278, 890, 924, 933, 934, 295, 563, 608], [266, 890]),
        (952, "Bindu", 0, [266, 268, 276, 277, 278, 890, 924, 933, 934, 295, 563, 608, 951], [951]),
        (953, "Tanmatra", 0, [951, 952, 954, 608, 563, 568, 890, 934, 554, 947, 955], [951, 952, 954]),
        (954, "Axiarchy", 0, [951, 952, 953, 955, 957, 958, 960, 963, 964, 965], [951, 952, 953]),
        (890, "World-Model-V3", 0, [266, 268, 276, 890, 924, 933, 934, 295, 563, 608], [266]),
        (923, "TemporalChain", 0, [255, 260, 261, 933, 262, 930, 912], [255, 260]),
        (933, "FluxMem", 0, [912, 913, 262, 255], [912, 913]),
        (939, "OmniAgent", 0, [940, 941, 942, 943, 944, 266, 268, 276], [940, 941]),
        (248, "Retrocausal-Caching", 0, [1, 900, 960, 965], [1, 900]),
    ]

    for sid, name, size, links, deps in substrates_data:
        meta.register_substrate(sid, name, size, links, deps)

    analysis = meta.analyze()
    print(meta.generate_report())

    return meta, analysis


if __name__ == "__main__":
    demo_self_reflexive()
