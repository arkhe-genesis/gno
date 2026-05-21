#!/usr/bin/env python3
"""
ARKHE OS — Substrato 412-SOURCE-CATALOG
API RESTful para consulta de fontes de raios cosmicos
Arquiteto: Rafael Oliveira (ORCID: 0009-0005-2697-4668)
"""

import json
import math
import time
from typing import List, Dict, Optional
from dataclasses import dataclass

# Simulacao de framework web (em producao: FastAPI + uvicorn)
class ArkheCatalogAPI:
    """
    API constitucional de consulta ao catalogo de fontes.
    Endpoints:
    - GET /sources — listar todas as fontes
    - GET /sources/{source_id} — detalhes de fonte especifica
    - GET /sources/search?ra={ra}&dec={dec}&radius={radius} — busca por coordenadas
    - GET /sources/class/{class_name} — filtrar por classe
    - GET /sources/energy?min={min}&max={max} — filtrar por energia
    - GET /sources/external/{catalog} — cross-match com catalogo externo
    - POST /sources/validate — validar candidato de fonte
    """

    def __init__(self, catalog_path: str = "arkhe_source_catalog.json"):
        self.catalog_path = catalog_path
        self.catalog = self._load_catalog()

    def _load_catalog(self) -> Dict:
        try:
            with open(self.catalog_path) as f:
                return json.load(f)
        except FileNotFoundError:
            return {"sources": []}

    def list_sources(self, limit: int = 100, offset: int = 0) -> Dict:
        """Lista fontes catalogadas com paginacao."""
        sources = self.catalog.get("sources", [])
        return {
            "total": len(sources),
            "limit": limit,
            "offset": offset,
            "sources": sources[offset:offset + limit],
        }

    def get_source(self, source_id: str) -> Optional[Dict]:
        """Obtem detalhes de uma fonte especifica."""
        for source in self.catalog.get("sources", []):
            if source["source_id"] == source_id:
                return source
        return None

    def search_by_coordinates(self, ra: float, dec: float, radius_deg: float = 1.0) -> List[Dict]:
        """Busca fontes dentro de um raio angular das coordenadas."""
        results = []
        for source in self.catalog.get("sources", []):
            dist = self._angular_distance(ra, dec, source["ra_deg"], source["dec_deg"])
            if dist <= radius_deg:
                results.append({**source, "angular_distance_deg": round(dist, 4)})
        return sorted(results, key=lambda x: x["angular_distance_deg"])

    def filter_by_class(self, class_name: str) -> List[Dict]:
        """Filtra fontes por classe (galactic, extragalactic, etc.)."""
        return [s for s in self.catalog.get("sources", []) if s["source_class"] == class_name]

    def filter_by_energy(self, min_kev: float = 0, max_kev: float = float('inf')) -> List[Dict]:
        """Filtra fontes por faixa de energia total."""
        return [
            s for s in self.catalog.get("sources", [])
            if min_kev <= s["total_energy_kev"] <= max_kev
        ]

    def cross_match_external(self, catalog: str) -> List[Dict]:
        """Retorna fontes com cross-match em catalogo externo."""
        return [
            s for s in self.catalog.get("sources", [])
            if catalog in s.get("external_catalog_ids", {})
        ]

    def validate_source_candidate(self, events: List[Dict]) -> Dict:
        """Valida se um conjunto de eventos constitui uma fonte."""
        if len(events) < 3:
            return {"valid": False, "reason": "Insuficiente eventos (min 3)"}

        # Verificar clustering espacial
        avg_lat = sum(e["latitude"] for e in events) / len(events)
        avg_lon = sum(e["longitude"] for e in events) / len(events)

        max_dist = max(
            self._angular_distance(avg_lat, avg_lon, e["latitude"], e["longitude"])
            for e in events
        )

        if max_dist > 2.0:
            return {"valid": False, "reason": "Eventos muito dispersos (>2°)"}

        # Verificar consistencia temporal
        timestamps = [e["timestamp_ns"] for e in events]
        time_span = max(timestamps) - min(timestamps)

        if time_span > 86400 * 1e9:  # > 24h
            return {"valid": False, "reason": "Span temporal muito grande (>24h)"}

        return {
            "valid": True,
            "centroid": {"lat": avg_lat, "lon": avg_lon},
            "max_spread_deg": round(max_dist, 4),
            "time_span_ns": time_span,
            "event_count": len(events),
        }

    def get_sky_map(self, resolution_deg: float = 5.0) -> Dict:
        """Gera mapa de densidade de fontes no ceu."""
        grid = {}
        for source in self.catalog.get("sources", []):
            ra_bin = int(source["ra_deg"] / resolution_deg) * resolution_deg
            dec_bin = int(source["dec_deg"] / resolution_deg) * resolution_deg
            key = f"{ra_bin}:{dec_bin}"
            grid[key] = grid.get(key, 0) + 1

        return {
            "resolution_deg": resolution_deg,
            "grid_cells": len(grid),
            "max_density": max(grid.values()) if grid else 0,
            "cells": [{"ra": k.split(":")[0], "dec": k.split(":")[1], "count": v} for k, v in grid.items()],
        }

    def _angular_distance(self, ra1: float, dec1: float, ra2: float, dec2: float) -> float:
        dra = math.radians(ra2 - ra1)
        ddec = math.radians(dec2 - dec1)
        a = math.sin(ddec/2)**2 + math.cos(math.radians(dec1)) * math.cos(math.radians(dec2)) * math.sin(dra/2)**2
        return math.degrees(2 * math.asin(math.sqrt(a)))


def main():
    print("=" * 70)
    print("ARKHE OS — API DE CATALOGACAO (Substrato 412)")
    print("=" * 70)

    api = ArkheCatalogAPI()

    print(f"\nCatalogo carregado: {len(api.catalog.get('sources', []))} fontes")

    # Demonstrar endpoints
    print("\n--- GET /sources (primeiras 5) ---")
    sources = api.list_sources(limit=5)
    for s in sources["sources"]:
        print(f"  {s['name']} ({s['source_class']}) @ RA={s['ra_deg']:.1f}, Dec={s['dec_deg']:.1f}")

    print("\n--- GET /sources/search?ra=180&dec=0&radius=30 ---")
    results = api.search_by_coordinates(ra=180, dec=0, radius_deg=30)
    print(f"  {len(results)} fontes encontradas")

    print("\n--- GET /sources/class/galactic ---")
    galactic = api.filter_by_class("galactic")
    print(f"  {len(galactic)} fontes galacticas")

    print("\n--- GET /sources/energy?min=1e9&max=1e12 ---")
    high_energy = api.filter_by_energy(min_kev=1e9, max_kev=1e12)
    print(f"  {len(high_energy)} fontes entre 1-1000 TeV")

    print("\n--- GET /sources/external/fermi_lat ---")
    fermi = api.cross_match_external("fermi_lat")
    print(f"  {len(fermi)} fontes com match Fermi LAT")

    print("\n--- GET /sky_map?resolution=10 ---")
    sky_map = api.get_sky_map(resolution_deg=10)
    print(f"  Células: {sky_map['grid_cells']}, Max densidade: {sky_map['max_density']}")

    print("\nPhi_C do catalogo:", api.catalog.get("phi_c", 0))
    print("Status:", "CANONIZED" if api.catalog.get("phi_c", 0) >= 0.95 else "REVIEW")


if __name__ == "__main__":
    main()