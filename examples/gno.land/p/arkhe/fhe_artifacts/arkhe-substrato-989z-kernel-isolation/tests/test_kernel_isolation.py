#!/usr/bin/env python3
"""
Testes canônicos — Substrato 989.z KERNEL-ISOLATION-ENGINE
Arquiteto ORCID: 0009-0005-2697-4668
Seal: 989.z-KERNEL-ISOLATION-ENGINE-B2C3D4E5F678901A
"""

import pytest
import asyncio
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from kernel_isolation_engine import (
    KernelIsolationEngine,
    IsolationDomain,
    IsolationModel,
    SecurityLevel,
    SyscallFilter,
)


# ===================================================================
# Fixtures
# ===================================================================

@pytest.fixture
def engine():
    return KernelIsolationEngine()


@pytest.fixture
def event_loop():
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()


# ===================================================================
# Testes de inicialização
# ===================================================================

def test_engine_constants(engine):
    assert engine.SUBSTRATE_ID == "989.z"
    assert engine.SEAL == "989.z-KERNEL-ISOLATION-ENGINE-B2C3D4E5F678901A"
    assert len(engine.domains) == 0
    assert engine.domain_counter == 0


def test_host_capabilities(engine):
    caps = engine.get_host_capabilities()
    assert "kvm" in caps
    assert "vtx_amd_v" in caps
    assert "nested_virtualization" in caps
    assert "recommended_models" in caps
    assert isinstance(caps["recommended_models"], list)


def test_syscall_filter_defaults(engine):
    sf = engine.syscall_filter
    assert len(sf.CATHEDRAL_ALLOWED) == 36
    assert len(sf.CATHEDRAL_DENIED) == 30
    assert "read" in sf.CATHEDRAL_ALLOWED
    assert "execve" in sf.CATHEDRAL_DENIED


# ===================================================================
# Testes de seleção de modelo
# ===================================================================

def test_select_model_trusted(engine):
    model = engine.select_isolation_model(SecurityLevel.TRUSTED)
    assert model == IsolationModel.NAMESPACE


def test_select_model_verified(engine):
    model = engine.select_isolation_model(SecurityLevel.VERIFIED)
    assert model in {IsolationModel.SECCOMP, IsolationModel.LVD}


def test_select_model_untrusted(engine):
    model = engine.select_isolation_model(SecurityLevel.UNTRUSTED)
    assert model in {IsolationModel.MICROVM, IsolationModel.KATA, IsolationModel.GVISOR}


def test_select_model_hostile(engine):
    model = engine.select_isolation_model(SecurityLevel.HOSTILE)
    assert model in {IsolationModel.MICROVM, IsolationModel.WASM}


def test_select_model_kernel_workload(engine):
    # Se tiver VT-x, workload kernel deve usar LVD
    model = engine.select_isolation_model(SecurityLevel.VERIFIED, workload_type="kernel")
    if engine.has_vtx:
        assert model == IsolationModel.LVD
    else:
        assert model == IsolationModel.SECCOMP


# ===================================================================
# Testes de criação de domínio
# ===================================================================

@pytest.mark.asyncio
async def test_create_domain_trusted(engine):
    domain = await engine.create_domain(
        security_level=SecurityLevel.TRUSTED,
        substrates=[1, 2, 3],
    )
    assert domain.domain_id.startswith("iso-")
    assert domain.model == IsolationModel.NAMESPACE
    assert domain.security_level == SecurityLevel.TRUSTED
    assert domain.cpu_cores == 4
    assert domain.memory_mb == 4096
    assert domain.disk_mb == 8192
    assert domain.substrates == [1, 2, 3]
    assert domain.is_running is True
    assert domain.seal.startswith("ISO-")
    assert len(domain.seal) == 20  # ISO- + 16 hex


@pytest.mark.asyncio
async def test_create_domain_verified(engine):
    domain = await engine.create_domain(
        security_level=SecurityLevel.VERIFIED,
        substrates=[954],
    )
    assert domain.security_level == SecurityLevel.VERIFIED
    assert domain.cpu_cores == 2
    assert domain.memory_mb == 2048
    assert domain.disk_mb == 4096


@pytest.mark.asyncio
async def test_create_domain_untrusted(engine):
    domain = await engine.create_domain(
        security_level=SecurityLevel.UNTRUSTED,
        substrates=[989],
    )
    assert domain.security_level == SecurityLevel.UNTRUSTED
    assert domain.cpu_cores == 1
    assert domain.memory_mb == 512


@pytest.mark.asyncio
async def test_create_domain_hostile(engine):
    domain = await engine.create_domain(
        security_level=SecurityLevel.HOSTILE,
        substrates=[999],
    )
    assert domain.security_level == SecurityLevel.HOSTILE
    assert domain.cpu_cores == 1
    assert domain.memory_mb == 256
    assert domain.disk_mb == 512


@pytest.mark.asyncio
async def test_create_domain_custom_resources(engine):
    domain = await engine.create_domain(
        security_level=SecurityLevel.TRUSTED,
        substrates=[1],
        custom_resources={"cpu": 8, "memory": 8192},
    )
    assert domain.cpu_cores == 8
    assert domain.memory_mb == 8192


@pytest.mark.asyncio
async def test_domain_counter_increment(engine):
    d1 = await engine.create_domain(SecurityLevel.TRUSTED, [1])
    d2 = await engine.create_domain(SecurityLevel.TRUSTED, [2])
    assert d1.domain_id != d2.domain_id
    assert engine.domain_counter == 2


# ===================================================================
# Testes de destruição de domínio
# ===================================================================

@pytest.mark.asyncio
async def test_destroy_domain(engine):
    domain = await engine.create_domain(SecurityLevel.TRUSTED, [1])
    domain_id = domain.domain_id
    assert domain_id in engine.domains

    result = await engine.destroy_domain(domain_id)
    assert result is True
    assert domain_id not in engine.domains


@pytest.mark.asyncio
async def test_destroy_nonexistent_domain(engine):
    result = await engine.destroy_domain("iso-999999-namespace")
    assert result is False


# ===================================================================
# Testes de relatório de domínio
# ===================================================================

@pytest.mark.asyncio
async def test_get_domain_report(engine):
    domain = await engine.create_domain(SecurityLevel.TRUSTED, [1, 2])
    report = engine.get_domain_report(domain.domain_id)
    assert report is not None
    assert report["domain_id"] == domain.domain_id
    assert report["model"] == "namespace"
    assert report["security_level"] == "trusted"
    assert report["resources"]["cpu"] == 4
    assert report["substrates"] == [1, 2]
    assert report["is_running"] is True
    assert "seal" in report


def test_get_domain_report_nonexistent(engine):
    report = engine.get_domain_report("iso-000000-namespace")
    assert report is None


# ===================================================================
# Testes de disponibilidade de modelo
# ===================================================================

def test_is_model_available_namespace(engine):
    assert engine._is_model_available(IsolationModel.NAMESPACE) is True


def test_is_model_available_seccomp(engine):
    assert engine._is_model_available(IsolationModel.SECCOMP) is True


def test_is_model_available_gvisor(engine):
    assert engine._is_model_available(IsolationModel.GVISOR) is True


def test_is_model_available_wasm(engine):
    assert engine._is_model_available(IsolationModel.WASM) is True


def test_is_model_available_microvm(engine):
    # Depende de KVM
    expected = engine.has_kvm
    assert engine._is_model_available(IsolationModel.MICROVM) == expected


def test_is_model_available_lvd(engine):
    # Depende de VT-x
    expected = engine.has_vtx
    assert engine._is_model_available(IsolationModel.LVD) == expected


# ===================================================================
# Testes de relatório canônico
# ===================================================================

def test_generate_report_empty(engine):
    report = engine.generate_report()
    assert "989.z-KERNEL-ISOLATION-ENGINE" in report
    assert "CANONIZED_PROVISIONAL" in report
    assert "Hephaestus" in report
    assert "Athena" in report
    assert "Nemesis" in report
    assert "Hecate" in report
    assert "HOST CAPABILITIES" in report
    assert "DOMAINS" in report


@pytest.mark.asyncio
async def test_generate_report_with_domains(engine):
    await engine.create_domain(SecurityLevel.TRUSTED, [1])
    await engine.create_domain(SecurityLevel.UNTRUSTED, [2])
    report = engine.generate_report()
    assert "Total: 2" in report
    assert "Running: 2" in report


# ===================================================================
# Testes de IsolationDomain
# ===================================================================

def test_domain_compute_seal():
    domain = IsolationDomain(
        domain_id="iso-000001-namespace",
        model=IsolationModel.NAMESPACE,
        security_level=SecurityLevel.TRUSTED,
        substrates=[1],
    )
    seal1 = domain.compute_seal()
    seal2 = domain.compute_seal()
    assert seal1.startswith("ISO-")
    assert seal1 == seal2  # Determinístico


def test_domain_seal_changes_with_substrates():
    d1 = IsolationDomain(
        domain_id="iso-000001-namespace",
        model=IsolationModel.NAMESPACE,
        security_level=SecurityLevel.TRUSTED,
        substrates=[1],
    )
    d2 = IsolationDomain(
        domain_id="iso-000001-namespace",
        model=IsolationModel.NAMESPACE,
        security_level=SecurityLevel.TRUSTED,
        substrates=[1, 2],
    )
    assert d1.compute_seal() != d2.compute_seal()


# ===================================================================
# Testes de SyscallFilter
# ===================================================================

def test_syscall_filter_empty():
    sf = SyscallFilter()
    assert sf.allowed_syscalls == []
    assert sf.denied_syscalls == []
    assert sf.audit_syscalls == []


def test_syscall_filter_class_defaults():
    assert "read" in SyscallFilter.CATHEDRAL_ALLOWED
    assert "execve" in SyscallFilter.CATHEDRAL_DENIED
    assert "bpf" in SyscallFilter.CATHEDRAL_DENIED


# ===================================================================
# Suite runner
# ===================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
