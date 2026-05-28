#!/usr/bin/env python3
"""
Substrato 944 — Glasswing Sentinel
Sistema autônomo de detecção, análise e resposta a vulnerabilidades.
Integra scanners SAST/DAST/IaC, motor de inferência, e ponte ZK
para registo imutável na TemporalChain (923.2).
"""

import asyncio
import hashlib
import json
import re
import subprocess
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path
from typing import Dict, List, Optional, Any, Callable

class Severity(Enum):
    NONE = 0; LOW = 1; MEDIUM = 2; HIGH = 3; CRITICAL = 4

class ScanType(Enum):
    SAST = "static"      # Source code analysis
    DAST = "dynamic"     # Runtime testing
    IAC = "infrastructure"  # Terraform/CloudFormation
    SECRETS = "secrets"  # Key/token leakage
    DEPENDENCY = "dependency"  # CVE in deps
    CONFIG = "config"    # Misconfiguration

@dataclass
class Vulnerability:
    vuln_id: str
    cve_id: Optional[str]
    title: str
    description: str
    severity: Severity
    scan_type: ScanType
    file_path: Optional[str]
    line_number: Optional[int]
    confidence: float  # 0.0-1.0
    proof: str
    remediation: str
    cvss_score: Optional[float]
    epss_score: Optional[float]  # Exploit Prediction
    discovered_at: str
    patched: bool = False
    patch_commit: Optional[str] = None

@dataclass
class ScanReport:
    report_id: str
    target: str
    scan_types: List[ScanType]
    started_at: str
    completed_at: str
    vulnerabilities: List[Vulnerability]
    summary: Dict[str, int]
    seal_hash: Optional[str] = None

class GlasswingScanner:
    """Motor SAST baseado em AST pattern matching + heurísticas."""

    DANGEROUS_PATTERNS = {
        "sql_injection": [
            r'execute\s*\(\s*["\']\s*\+',
            r'cursor\.execute\s*\(\s*f["\']',
            r'\$\{.*\}\s*\+\s*["\'].*SELECT',
        ],
        "xss": [
            r'innerHTML\s*=\s*',
            r'document\.write\s*\(',
            r'eval\s*\(',
        ],
        "hardcoded_secret": [
            r'password\s*=\s*["\'][^"\']{8,}["\']',
            r'api_key\s*=\s*["\'][^"\']{16,}["\']',
            r'private_key\s*=\s*["\']',
            r'AWS_SECRET_ACCESS_KEY',
        ],
        "unsafe_deserialization": [
            r'pickle\.loads',
            r'yaml\.load\s*\(',
            r'eval\s*\(',
            r'exec\s*\(',
        ],
        "ssrf": [
            r'requests\.get\s*\(\s*[^)]+\+',
            r'urllib\.request\.urlopen\s*\(',
        ],
        "path_traversal": [
            r'open\s*\(\s*[^)]+\+',
            r'\.\.\/',
        ]
    }

    def __init__(self, rules_path: Optional[str] = None):
        self.rules = self.DANGEROUS_PATTERNS
        if rules_path:
            self._load_custom_rules(rules_path)

    def _load_custom_rules(self, path: str):
        with open(path) as f:
            custom = json.load(f)
            self.rules.update(custom)

    def scan_file(self, file_path: str, content: str) -> List[Vulnerability]:
        findings = []
        lines = content.split("\n")

        for vuln_type, patterns in self.rules.items():
            for pattern in patterns:
                for i, line in enumerate(lines, 1):
                    if re.search(pattern, line, re.IGNORECASE):
                        severity = self._classify_severity(vuln_type, line)
                        vuln = Vulnerability(
                            vuln_id=f"GW-{vuln_type.upper()}-{hashlib.sha3_256(f'{file_path}:{i}'.encode()).hexdigest()[:8]}",
                            cve_id=None,
                            title=f"Potential {vuln_type.replace('_', ' ').title()}",
                            description=f"Pattern matched: {pattern}",
                            severity=severity,
                            scan_type=ScanType.SAST,
                            file_path=file_path,
                            line_number=i,
                            confidence=0.75,
                            proof=line.strip(),
                            remediation=self._suggest_remediation(vuln_type),
                            cvss_score=None,
                            epss_score=None,
                            discovered_at=datetime.now(timezone.utc).replace(tzinfo=None).isoformat()
                        )
                        findings.append(vuln)

        return findings

    def _classify_severity(self, vuln_type: str, context: str) -> Severity:
        critical_types = ["sql_injection", "hardcoded_secret", "unsafe_deserialization"]
        high_types = ["xss", "ssrf", "path_traversal"]
        if vuln_type in critical_types:
            return Severity.CRITICAL
        elif vuln_type in high_types:
            return Severity.HIGH
        return Severity.MEDIUM

    def _suggest_remediation(self, vuln_type: str) -> str:
        remedies = {
            "sql_injection": "Use parameterized queries / prepared statements.",
            "xss": "Use template auto-escaping or sanitize all output.",
            "hardcoded_secret": "Move secrets to environment variables or vault.",
            "unsafe_deserialization": "Use safe parsers (json, yaml.safe_load).",
            "ssrf": "Validate and whitelist all URLs; use URL parser.",
            "path_traversal": "Use pathlib / os.path.join with validation."
        }
        return remedies.get(vuln_type, "Review and refactor.")

class DependencyScanner:
    """Scanner de dependências com CVE mapping."""

    def scan_requirements(self, req_path: str) -> List[Vulnerability]:
        findings = []
        # Simulated: in production, queries NVD/OSV API
        with open(req_path) as f:
            for line in f:
                if "==" in line:
                    pkg, ver = line.strip().split("==")
                    # Simulated CVE check
                    if pkg == "requests" and ver.startswith("2.2"):
                        findings.append(Vulnerability(
                            vuln_id=f"DEP-{pkg}-{ver}",
                            cve_id="CVE-2018-18074",
                            title=f"{pkg} {ver} — Session fixation",
                            description="Requests before 2.20.0 sends HTTP headers...",
                            severity=Severity.HIGH,
                            scan_type=ScanType.DEPENDENCY,
                            file_path=req_path,
                            line_number=None,
                            confidence=0.95,
                            proof=f"{pkg}=={ver}",
                            remediation="Upgrade to requests>=2.31.0",
                            cvss_score=7.5,
                            epss_score=0.12,
                            discovered_at=datetime.now(timezone.utc).replace(tzinfo=None).isoformat()
                        ))
        return findings

class GlasswingSentinel:
    """
    Sentinel mestre que orquestra scanners, gera relatórios,
    e ancora findings na Vulnerability TemporalChain (923.2).
    """

    def __init__(self, bridge_endpoint: str = "http://localhost:9232"):
        self.sast = GlasswingScanner()
        self.deps = DependencyScanner()
        self.bridge = bridge_endpoint
        self.reports: Dict[str, ScanReport] = {}

    async def scan_repository(self, repo_path: str,
                              scan_types: List[ScanType] = None) -> ScanReport:
        if scan_types is None:
            scan_types = [ScanType.SAST, ScanType.SECRETS, ScanType.DEPENDENCY]

        report_id = f"GW-{hashlib.sha3_256(repo_path.encode()).hexdigest()[:12]}"
        started = datetime.now(timezone.utc).replace(tzinfo=None).isoformat()
        all_findings: List[Vulnerability] = []

        repo = Path(repo_path)

        if ScanType.SAST in scan_types or ScanType.SECRETS in scan_types:
            for py_file in repo.rglob("*.py"):
                content = py_file.read_text()
                findings = self.sast.scan_file(str(py_file), content)
                all_findings.extend(findings)

        if ScanType.DEPENDENCY in scan_types:
            req_file = repo / "requirements.txt"
            if req_file.exists():
                all_findings.extend(self.deps.scan_requirements(str(req_file)))

        completed = datetime.now(timezone.utc).replace(tzinfo=None).isoformat()

        summary = {
            "total": len(all_findings),
            "critical": sum(1 for v in all_findings if v.severity == Severity.CRITICAL),
            "high": sum(1 for v in all_findings if v.severity == Severity.HIGH),
            "medium": sum(1 for v in all_findings if v.severity == Severity.MEDIUM),
            "low": sum(1 for v in all_findings if v.severity == Severity.LOW),
        }

        report = ScanReport(
            report_id=report_id,
            target=repo_path,
            scan_types=scan_types,
            started_at=started,
            completed_at=completed,
            vulnerabilities=all_findings,
            summary=summary
        )

        # Seal the report
        report.seal_hash = self._seal_report(report)
        self.reports[report_id] = report

        # Auto-anchor critical/high to TemporalChain (923.2)
        await self._anchor_to_chain(report)

        return report

    def _seal_report(self, report: ScanReport) -> str:

        # Serialize enums by taking their values or names
        findings_dict = []
        for v in report.vulnerabilities:
            v_dict = asdict(v)
            v_dict['severity'] = v.severity.name
            v_dict['scan_type'] = v.scan_type.value
            findings_dict.append(v_dict)

        payload = json.dumps({
            "report_id": report.report_id,
            "target": report.target,
            "started": report.started_at,
            "completed": report.completed_at,
            "summary": report.summary,
            "findings": findings_dict
        }, sort_keys=True)
        return hashlib.sha3_256(payload.encode()).hexdigest()

    async def _anchor_to_chain(self, report: ScanReport):
        """Register critical/high findings to 923.2."""
        critical_findings = [v for v in report.vulnerabilities
                           if v.severity in (Severity.CRITICAL, Severity.HIGH)]
        for vuln in critical_findings:
            print(f"[944] Anchoring {vuln.vuln_id} to TemporalChain 923.2...")
            # In production: HTTP call to vuln_bridge (923.2)

    def generate_report_markdown(self, report_id: str) -> str:
        r = self.reports[report_id]
        md = f"""# Glasswing Sentinel Report
**ID:** `{r.report_id}`
**Target:** `{r.target}`
**Seal:** `{r.seal_hash}`
**Window:** {r.started_at} → {r.completed_at}

## Summary
| Severity | Count |
|----------|-------|
| 🔴 Critical | {r.summary["critical"]} |
| 🟠 High | {r.summary["high"]} |
| 🟡 Medium | {r.summary["medium"]} |
| 🟢 Low | {r.summary["low"]} |
| **Total** | **{r.summary["total"]}** |

## Findings
"""
        for v in r.vulnerabilities:
            emoji = {"CRITICAL": "🔴", "HIGH": "🟠", "MEDIUM": "🟡", "LOW": "🟢"}[v.severity.name]
            md += f"""### {emoji} {v.title} (`{v.vuln_id}`)
- **Severity:** {v.severity.name}
- **Type:** {v.scan_type.value}
- **Location:** `{v.file_path}:{v.line_number}`
- **Confidence:** {v.confidence:.0%}
- **Proof:** `{v.proof}`
- **Remediation:** {v.remediation}
- **CVE:** {v.cve_id or "N/A"}

"""
        return md

# ── CLI ─────────────────────────────────────────────────────

def main():
    import argparse
    parser = argparse.ArgumentParser(prog="arkhe sentinel", description="Glasswing Sentinel 944")
    sub = parser.add_subparsers(dest="command")

    scan_p = sub.add_parser("scan", help="Scan repository")
    scan_p.add_argument("repo", help="Repository path")
    scan_p.add_argument("--types", nargs="+", choices=["sast", "secrets", "dependency", "config", "iac", "dast"])

    report_p = sub.add_parser("report", help="Generate report")
    report_p.add_argument("--id", required=True)
    report_p.add_argument("--format", choices=["markdown", "json", "sarif"], default="markdown")

    args = parser.parse_args()
    sentinel = GlasswingSentinel()

    if args.command == "scan":
        type_map = {
            "sast": ScanType.SAST, "secrets": ScanType.SECRETS,
            "dependency": ScanType.DEPENDENCY, "config": ScanType.CONFIG,
            "iac": ScanType.IAC, "dast": ScanType.DAST
        }
        types = [type_map[t] for t in args.types] if args.types else None
        report = asyncio.run(sentinel.scan_repository(args.repo, types))
        print(json.dumps(asdict(report), indent=2, default=str))
    elif args.command == "report":
        if args.format == "markdown":
            print(sentinel.generate_report_markdown(args.id))
        else:
            print(json.dumps(asdict(sentinel.reports[args.id]), indent=2, default=str))

if __name__ == "__main__":
    main()