#!/usr/bin/env python3
"""
Testes canonicos — Substrato 989.x PASSPORT-GATEWAY
Arquiteto ORCID: 0009-0005-2697-4668
Seal: 989-PASSPORT-GATEWAY-4B3CB68C02D21E5A
"""

import pytest
from unittest.mock import AsyncMock, MagicMock
from aiohttp import ClientSession

import sys
import os
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
sys.path.insert(0, "..")
from passport_gateway import (
    PassportGateway,
    PassportGatewayError,
    HumanityProof,
    StampCredential,
    VerificationStatus,
    MIN_HUMANITY_SCORE,
    MIN_PASSPORT_SCORE,
)


# ===================================================================
# Fixtures
# ===================================================================

@pytest.fixture
def gateway():
    g = PassportGateway(api_key="test-key", scorer_id="42")
    return g


@pytest.fixture
def mock_session():
    session = AsyncMock(spec=ClientSession)
    return session


# ===================================================================
# Helpers
# ===================================================================

def make_mock_response(status, json_data):
    """Cria um mock de resposta aiohttp."""
    class MockResponse:
        def __init__(self, status, json_data):
            self.status = status
            self._json_data = json_data

        async def json(self):
            return self._json_data

        async def text(self):
            return str(self._json_data)

    mock_resp = MockResponse(status, json_data)

    class MockContextManager:
        def __init__(self, resp):
            self.resp = resp

        async def __aenter__(self):
            return self.resp

        async def __aexit__(self, exc_type, exc, tb):
            pass

    return MockContextManager(mock_resp)


def build_credential(provider, issuance_date=None):
    """Constroi um credential do Passport de forma segura."""
    cred = {"credentialSubject": {"provider": provider}}
    if issuance_date:
        cred["issuanceDate"] = issuance_date
    return {"credential": cred}


# ===================================================================
# Testes de inicializacao
# ===================================================================

@pytest.mark.asyncio
async def test_gateway_start_stop(gateway):
    await gateway.start()
    assert gateway._session is not None
    await gateway.stop()


def test_gateway_constants():
    assert PassportGateway.SUBSTRATE_ID == 989
    assert PassportGateway.VARIANT == "x"
    assert PassportGateway.SEAL == "989-PASSPORT-GATEWAY-4B3CB68C02D21E5A"


def test_humanity_proof_seal():
    proof = HumanityProof(
        address="0xTest",
        is_human=True,
        score=0.85,
        raw_passport_score=17.0,
        stamps=[],
        orcid_verified=True,
        status=VerificationStatus.VERIFIED,
    )
    seal = proof.compute_seal()
    assert seal.startswith("HP-")
    assert len(seal) == 19
    assert proof.signature != ""
    assert proof.temporal_anchor.startswith("923-resp-")
    seal2 = proof.compute_seal()
    assert seal == seal2


def test_humanity_proof_to_dict():
    proof = HumanityProof(
        address="0xTest",
        is_human=True,
        score=0.85,
        raw_passport_score=17.0,
        stamps=[StampCredential(provider="Google")],
        orcid_verified=True,
        status=VerificationStatus.VERIFIED,
    )
    d = proof.to_dict()
    assert d["address"] == "0xTest"
    assert d["is_human"] is True
    assert d["status"] == "verified"
    assert len(d["stamps"]) == 1
    assert d["stamps"][0]["provider"] == "Google"


# ===================================================================
# Testes de API Passport (mockados)
# ===================================================================

@pytest.mark.asyncio
async def test_get_passport_score_success(gateway, mock_session):
    resp = make_mock_response(200, {"score": "25.5", "status": "done"})
    mock_session.get = MagicMock(return_value=resp)
    gateway._session = mock_session

    result = await gateway.get_passport_score("0xAlice")
    assert result["score"] == "25.5"
    mock_session.get.assert_called_once()


@pytest.mark.asyncio
async def test_get_passport_score_404(gateway, mock_session):
    resp = make_mock_response(404, {})
    mock_session.get = MagicMock(return_value=resp)
    gateway._session = mock_session

    result = await gateway.get_passport_score("0xUnknown")
    assert result["score"] == 0
    assert result["status"] == "NOT_FOUND"


@pytest.mark.asyncio
async def test_get_passport_score_no_api_key():
    g = PassportGateway(api_key="", scorer_id="1")
    with pytest.raises(PassportGatewayError, match="PASSPORT_API_KEY"):
        await g.get_passport_score("0xAlice")


@pytest.mark.asyncio
async def test_get_passport_stamps(gateway, mock_session):
    data = {
        "items": [
            build_credential("Google", "2026-01-01"),
            build_credential("GitHub"),
        ]
    }
    resp = make_mock_response(200, data)
    mock_session.get = MagicMock(return_value=resp)
    gateway._session = mock_session

    stamps = await gateway.get_passport_stamps("0xAlice")
    assert len(stamps) == 2
    assert stamps[0].provider == "Google"
    assert stamps[0].issuance_date == "2026-01-01"
    assert stamps[1].provider == "GitHub"


@pytest.mark.asyncio
async def test_get_passport_stamps_empty(gateway, mock_session):
    resp = make_mock_response(200, {"items": []})
    mock_session.get = MagicMock(return_value=resp)
    gateway._session = mock_session

    stamps = await gateway.get_passport_stamps("0xEmpty")
    assert stamps == []


# ===================================================================
# Testes de verificacao de humanidade
# ===================================================================

@pytest.mark.asyncio
async def test_is_human_high_score(gateway, mock_session):
    score_resp = make_mock_response(200, {"score": "30.0"})
    stamps_resp = make_mock_response(200, {"items": []})
    mock_session.get = MagicMock(side_effect=[score_resp, stamps_resp])
    gateway._session = mock_session

    proof = await gateway.is_human("0xAlice")
    assert proof.is_human is True
    assert proof.score == 1.0
    assert proof.raw_passport_score == 30.0
    assert proof.status == VerificationStatus.VERIFIED
    assert proof.seal.startswith("HP-")


@pytest.mark.asyncio
async def test_is_human_low_score_no_orcid(gateway, mock_session):
    score_resp = make_mock_response(200, {"score": "5.0"})
    stamps_resp = make_mock_response(200, {"items": []})
    mock_session.get = MagicMock(side_effect=[score_resp, stamps_resp])
    gateway._session = mock_session

    proof = await gateway.is_human("0xBob")
    assert proof.is_human is False
    assert proof.score == 0.25
    assert proof.orcid_verified is False


@pytest.mark.asyncio
async def test_is_human_orcid_fallback(gateway, mock_session):
    score_resp = make_mock_response(200, {"score": "5.0"})
    stamps_resp = make_mock_response(200, {"items": []})
    mock_session.get = MagicMock(side_effect=[score_resp, stamps_resp])
    gateway._session = mock_session

    proof = await gateway.is_human("0xAlice123456789")
    assert proof.is_human is True
    assert proof.orcid_verified is True


@pytest.mark.asyncio
async def test_is_human_api_error_with_orcid(gateway, mock_session):
    score_resp = make_mock_response(500, {})
    score_resp.text = AsyncMock(return_value="Internal Server Error")
    orcid_resp = make_mock_response(200, {"orcid-identifier": {"path": "0009-0005-2697-4668"}})
    mock_session.get = MagicMock(side_effect=[score_resp, orcid_resp])
    gateway._session = mock_session

    proof = await gateway.is_human("0xAlice123456789", orcid_id="0009-0005-2697-4668")
    assert proof.status == VerificationStatus.PENDING
    assert proof.orcid_verified is True

@pytest.mark.asyncio
async def test_cache_ttl(gateway, mock_session):
    score_resp = make_mock_response(200, {"score": "30.0"})
    stamps_resp = make_mock_response(200, {"items": []})
    mock_session.get = MagicMock(side_effect=[score_resp, stamps_resp])
    gateway._session = mock_session

    proof1 = await gateway.is_human("0xAlice")
    assert "0xAlice" in gateway.cache
    proof2 = await gateway.is_human("0xAlice")

    # The second call shouldn't exhaust mock_session (it only had 2 responses set)
    assert proof1 == proof2

# ===================================================================
# Testes de governanca DAO (979)
# ===================================================================

@pytest.mark.asyncio
async def test_verify_dao_voter_human(gateway, mock_session):
    score_resp = make_mock_response(200, {"score": "25.0"})
    stamps_resp = make_mock_response(200, {"items": []})
    mock_session.get = MagicMock(side_effect=[score_resp, stamps_resp])
    gateway._session = mock_session

    can_vote = await gateway.verify_dao_voter("0xAlice")
    assert can_vote is True


@pytest.mark.asyncio
async def test_verify_dao_voter_sanctions(gateway, mock_session):
    score_resp = make_mock_response(200, {"score": "25.0"})
    stamps_resp = make_mock_response(200, {"items": []})
    mock_session.get = MagicMock(side_effect=[score_resp, stamps_resp])
    gateway._session = mock_session

    can_vote = await gateway.verify_dao_voter("0xSanctioned123...")
    # Can vote handles sanctions clear in verify_dao_voter (returns proof.is_human and proof.sanctions_clear)
    assert can_vote is False


# ===================================================================
# Testes de malha global (972)
# ===================================================================

@pytest.mark.asyncio
async def test_verify_node_access(gateway, mock_session):
    score_resp = make_mock_response(200, {"score": "25.0"})
    stamps_resp = make_mock_response(200, {"items": []})
    mock_session.get = MagicMock(side_effect=[score_resp, stamps_resp])
    gateway._session = mock_session

    can_operate = await gateway.verify_node_access("0xAlice")
    assert can_operate is True


# ===================================================================
# Testes Axiarchy (954)
# ===================================================================

@pytest.mark.asyncio
async def test_axiarchy_validate_vote(gateway, mock_session):
    score_resp = make_mock_response(200, {"score": "25.0"})
    stamps_resp = make_mock_response(200, {"items": []})
    mock_session.get = MagicMock(side_effect=[score_resp, stamps_resp])
    gateway._session = mock_session

    result = await gateway.axiarchy_validate("0xAlice", "vote")
    assert result["approved"] is True
    assert result["action"] == "vote"
    assert result["substrate"] == "989.x"
    assert "seal" in result


@pytest.mark.asyncio
async def test_axiarchy_validate_rejected(gateway, mock_session):
    score_resp = make_mock_response(200, {"score": "5.0"})
    stamps_resp = make_mock_response(200, {"items": []})
    mock_session.get = MagicMock(side_effect=[score_resp, stamps_resp])
    gateway._session = mock_session

    result = await gateway.axiarchy_validate("0xSybil", "treasury")
    assert result["approved"] is False
    assert result["humanity_score"] == 0.25


# ===================================================================
# Testes de ORCID (982)
# ===================================================================

@pytest.mark.asyncio
async def test_verify_orcid_link_fallback(gateway):
    assert await gateway.verify_orcid_link("0xAlice123") is True
    assert await gateway.verify_orcid_link("0xArchitect0009") is True
    assert await gateway.verify_orcid_link("0xBob456") is False


@pytest.mark.asyncio
async def test_get_orcid_record(gateway, mock_session):
    data = {
        "orcid-identifier": {"path": "0009-0005-2697-4668"},
        "person": {"name": {"given-names": {"value": "Arquiteto"}}}
    }
    resp = make_mock_response(200, data)
    mock_session.get = MagicMock(return_value=resp)
    gateway._session = mock_session

    record = await gateway.get_orcid_record("0009-0005-2697-4668")
    assert record["orcid-identifier"]["path"] == "0009-0005-2697-4668"


@pytest.mark.asyncio
async def test_get_orcid_record_404(gateway, mock_session):
    resp = make_mock_response(404, {})
    mock_session.get = MagicMock(return_value=resp)
    gateway._session = mock_session

    record = await gateway.get_orcid_record("0000-0000-0000-0000")
    assert record == {}


# ===================================================================
# Testes de relatorio canonico
# ===================================================================

def test_generate_report(gateway):
    report = gateway.generate_report()
    assert "989-PASSPORT-GATEWAY-4B3CB68C02D21E5A" in report
    assert "CANONIZED_PROVISIONAL" in report
    assert "Themis" in report
    assert "Athena" in report
    assert "Hermes" in report
    assert "979" in report
    assert "954" in report


# ===================================================================
# Testes de edge cases e resiliencia
# ===================================================================

@pytest.mark.asyncio
async def test_is_human_zero_score(gateway, mock_session):
    score_resp = make_mock_response(200, {"score": "0"})
    stamps_resp = make_mock_response(200, {"items": []})
    mock_session.get = MagicMock(side_effect=[score_resp, stamps_resp])
    gateway._session = mock_session

    proof = await gateway.is_human("0xZero")
    assert proof.score == 0.0
    assert proof.is_human is False


@pytest.mark.asyncio
async def test_is_human_exact_threshold(gateway, mock_session):
    score_resp = make_mock_response(200, {"score": "15.0"})
    stamps_resp = make_mock_response(200, {"items": []})
    mock_session.get = MagicMock(side_effect=[score_resp, stamps_resp])
    gateway._session = mock_session

    proof = await gateway.is_human("0xThreshold")
    assert proof.score == 0.75
    assert proof.is_human is True


@pytest.mark.asyncio
async def test_multiple_stamps_parsing(gateway, mock_session):
    score_resp = make_mock_response(200, {"score": "30.0"})
    stamps_data = {
        "items": [
            build_credential("Google", "2026-01-01"),
            build_credential("Twitter", "2026-02-01"),
            build_credential("GitHub", "2026-03-01"),
            build_credential("LinkedIn"),
        ]
    }
    stamps_resp = make_mock_response(200, stamps_data)
    mock_session.get = MagicMock(side_effect=[score_resp, stamps_resp])
    gateway._session = mock_session

    proof = await gateway.is_human("0xRich")
    assert len(proof.stamps) == 4
    providers = [s.provider for s in proof.stamps]
    assert "Google" in providers
    assert "Twitter" in providers
    assert "GitHub" in providers
    assert "LinkedIn" in providers

# ===================================================================
# Suite runner
# ===================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
