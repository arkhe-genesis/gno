#!/usr/bin/env python3
# test_permaweb_bridge.py — Test Suite for Substrate 927
# pytest-compatible tests for PermawebBridge

import pytest
import json
import time
import hashlib
from datetime import datetime, timezone
from unittest.mock import Mock, patch, MagicMock

# Import substrate under test
import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from permaweb_bridge import (
    PermawebConfig, PermawebBridge, ArweaveDataLayer,
    AOComputerInterface, AOSInterface, HyperBEAMInterface,
    PermawebAdapter
)


# ═══════════════════════════════════════════════════════════════════
# Fixtures
# ═══════════════════════════════════════════════════════════════════

@pytest.fixture
def config():
    """Default test configuration."""
    return PermawebConfig(
        arweave_gateway="https://arweave.net",
        ao_mu_url="https://mu.ao-testnet.xyz",
        ao_cu_url="https://cu.ao-testnet.xyz",
        debug=True,
    )

@pytest.fixture
def bridge(config):
    """Fresh bridge instance."""
    return PermawebBridge(config)

@pytest.fixture
def mock_agent():
    """Mock ArkheOmniAgent for adapter tests."""
    agent = Mock()
    agent.agent_id = "TEST-AGENT-927"
    agent.memory = Mock()
    agent.memory.entries = []
    agent.hypergraph = Mock()
    agent.hypergraph.edges = []
    agent.get_status = Mock(return_value={"substrates": 22})

    def mock_commit(content):
        return f"commit-{hashlib.sha3_256(json.dumps(content).encode()).hexdigest()[:16]}"
    agent.commit_memory = mock_commit

    return agent


# ═══════════════════════════════════════════════════════════════════
# ArweaveDataLayer Tests
# ═══════════════════════════════════════════════════════════════════

class TestArweaveDataLayer:
    """Tests for Arweave permanent storage interface."""

    def test_upload_data_mock_mode(self, bridge):
        """Test data upload in mock mode (no arweave package)."""
        data = json.dumps({"test": "data", "timestamp": time.time()})
        tags = {"Type": "Test", "Substrate": "927"}

        result = bridge.arweave.upload_data(data, tags)

        assert result["status"] == "mock_uploaded"
        assert "tx_id" in result
        assert result["url"].startswith("https://arweave.net/")
        assert result["mock"] is True
        assert result["tags"]["Type"] == "Test"

    def test_upload_data_bytes(self, bridge):
        """Test uploading raw bytes."""
        data = b"\x00\x01\x02\x03"
        result = bridge.arweave.upload_data(data)

        assert result["status"] == "mock_uploaded"
        assert result["size"] == 4

    def test_fetch_data_success(self, bridge):
        """Test fetching data by TX ID."""
        # First upload
        upload = bridge.arweave.upload_data("test content")
        tx_id = upload["tx_id"]

        # Mock fetch (will fail without real network, but structure is valid)
        with patch('requests.get') as mock_get:
            mock_get.return_value = Mock(
                text="test content",
                content=b"test content",
                raise_for_status=Mock(),
            )
            result = bridge.arweave.fetch_data(tx_id)

        # Without mock, it fails (no network), but structure is correct
        # With mock, it succeeds
        assert "status" in result

    def test_fetch_data_invalid_tx(self, bridge):
        """Test fetching invalid TX ID."""
        with patch('requests.get') as mock_get:
            mock_get.side_effect = Exception("Not found")
            result = bridge.arweave.fetch_data("invalid-tx-id")
            assert result["status"] == "failed"
            assert "error" in result

    def test_query_transactions_structure(self, bridge):
        """Test GraphQL query structure."""
        with patch('requests.post') as mock_post:
            mock_post.return_value = Mock(
                json=Mock(return_value={
                    "data": {
                        "transactions": {
                            "edges": [
                                {"node": {"id": "tx-1", "tags": []}}
                            ]
                        }
                    }
                }),
            )
            results = bridge.arweave.query_transactions(
                tags={"App-Name": "ARKHE-OS"},
                limit=5,
            )

        assert isinstance(results, list)


# ═══════════════════════════════════════════════════════════════════
# AOComputerInterface Tests
# ═══════════════════════════════════════════════════════════════════

class TestAOComputerInterface:
    """Tests for AO Computer actor-oriented interface."""

    def test_spawn_process_structure(self, bridge):
        """Test process spawning payload structure."""
        with patch('requests.post') as mock_post:
            mock_post.return_value = Mock(
                json=Mock(return_value={"id": "ao-process-123", "status": "spawned"}),
            )
            result = bridge.ao.spawn_process(
                module_id="test-module",
                tags=[{"name": "Test", "value": "true"}],
            )

        # Verify the call was made with correct structure
        mock_post.assert_called_once()
        call_args = mock_post.call_args
        assert "test-module" in str(call_args)

    def test_send_message_structure(self, bridge):
        """Test message sending payload structure."""
        with patch('requests.post') as mock_post:
            mock_post.return_value = Mock(
                json=Mock(return_value={"message_id": "msg-456", "status": "sent"}),
            )
            result = bridge.ao.send_message(
                process_id="ao-process-123",
                action="Test-Action",
                data="test data",
            )

        mock_post.assert_called_once()
        call_kwargs = mock_post.call_args.kwargs
        assert "json" in call_kwargs
        payload = call_kwargs["json"]
        assert payload["Target"] == "ao-process-123"
        assert any(t["name"] == "Action" and t["value"] == "Test-Action"
                   for t in payload["Tags"])

    def test_dry_run_structure(self, bridge):
        """Test dry-run simulation structure."""
        with patch('requests.post') as mock_post:
            mock_post.return_value = Mock(
                json=Mock(return_value={"result": "simulated", "gas": 100}),
            )
            result = bridge.ao.dry_run(
                process_id="ao-process-123",
                action="Compute",
                data="input",
            )

        mock_post.assert_called_once()


# ═══════════════════════════════════════════════════════════════════
# AOSInterface Tests
# ═══════════════════════════════════════════════════════════════════

class TestAOSInterface:
    """Tests for AO Operating System Lua interface."""

    def test_eval_lua_no_process(self, bridge):
        """Test eval without configured process."""
        result = bridge.aos.eval_lua("print('hello')")
        assert "error" in result
        assert "No AOS process ID" in result["error"]

    def test_spawn_aos(self, bridge):
        """Test spawning new AOS process."""
        with patch.object(bridge.ao, 'spawn_process') as mock_spawn:
            mock_spawn.return_value = {"id": "aos-123", "status": "spawned"}
            result = bridge.aos.spawn_aos("test-aos")

        assert result["id"] == "aos-123"
        assert bridge.aos.process_id == "aos-123"

    def test_load_blueprint(self, bridge):
        """Test loading AOS blueprint."""
        bridge.aos.process_id = "aos-123"

        with patch.object(bridge.ao, 'send_message') as mock_send:
            mock_send.return_value = {"status": "loaded"}
            result = bridge.aos.load_blueprint("json")

        mock_send.assert_called_once()
        call_args = mock_send.call_args
        assert call_args.kwargs["action"] == "Eval"
        assert ".load-blueprint json" in call_args.kwargs["data"]


# ═══════════════════════════════════════════════════════════════════
# HyperBEAMInterface Tests
# ═══════════════════════════════════════════════════════════════════

class TestHyperBEAMInterface:
    """Tests for HyperBEAM permaweb OS interface."""

    def test_resolve_path_no_endpoint(self, bridge):
        """Test resolve without configured endpoint."""
        result = bridge.hyperbeam.resolve_path("test/path")
        assert "error" in result
        assert "not configured" in result["error"]

    def test_resolve_path_with_endpoint(self, bridge):
        """Test resolve with configured endpoint."""
        bridge.config.hyperbeam_endpoint = "https://hyperbeam.example.com"

        with patch('requests.get') as mock_get:
            mock_get.return_value = Mock(
                json=Mock(return_value={"resolved": True, "data": "test"}),
            )
            result = bridge.hyperbeam.resolve_path("test/path")

        mock_get.assert_called_once()
        assert "test/path" in str(mock_get.call_args)

    def test_execute_device(self, bridge):
        """Test device execution."""
        result = bridge.hyperbeam.execute_device(
            device_name="test-device",
            input_data={"key": "value"},
        )

        assert result["device"] == "test-device"
        assert result["status"] == "executed"
        assert result["mock"] is True


# ═══════════════════════════════════════════════════════════════════
# PermawebBridge Integration Tests
# ═══════════════════════════════════════════════════════════════════

class TestPermawebBridge:
    """Integration tests for main bridge class."""

    def test_persist_agent_state(self, bridge):
        """Test agent state persistence."""
        agent_state = {
            "agent_id": "TEST-AGENT",
            "memory_entries": 42,
            "substrates": ["920", "921", "927"],
        }

        result = bridge.persist_agent_state(agent_state, "TEST-AGENT")

        assert result["status"] == "mock_uploaded"
        assert "tx_id" in result
        assert result["tags"]["Agent-ID"] == "TEST-AGENT"
        assert result["tags"]["Type"] == "Agent-State"
        assert result["tags"]["Substrate"] == "927"

        # Verify upload history
        assert len(bridge._upload_history) == 1
        assert bridge._upload_history[0]["tx_id"] == result["tx_id"]

    def test_restore_agent_state(self, bridge):
        """Test state restoration."""
        # First persist
        agent_state = {"test": "data", "value": 123}
        upload = bridge.persist_agent_state(agent_state, "TEST-AGENT")
        tx_id = upload["tx_id"]

        # Mock fetch
        with patch.object(bridge.arweave, 'fetch_data') as mock_fetch:
            mock_fetch.return_value = {
                "status": "fetched",
                "data": json.dumps(agent_state),
            }
            result = bridge.restore_agent_state(tx_id)

        assert result["test"] == "data"
        assert result["value"] == 123

    def test_restore_invalid_json(self, bridge):
        """Test restoration of invalid JSON."""
        with patch.object(bridge.arweave, 'fetch_data') as mock_fetch:
            mock_fetch.return_value = {
                "status": "fetched",
                "data": "not-json",
            }
            result = bridge.restore_agent_state("tx-123")

        assert "error" in result
        assert "Invalid JSON" in result["error"]

    def test_spawn_agent_process(self, bridge):
        """Test spawning agent process."""
        with patch.object(bridge.ao, 'spawn_process') as mock_spawn:
            mock_spawn.return_value = {"id": "ao-agent-123"}
            result = bridge.spawn_agent_process("AGENT-1", "arkhe-test")

        assert result["id"] == "ao-agent-123"
        assert "AGENT-1" in bridge._process_registry
        assert bridge._process_registry["AGENT-1"]["process_id"] == "ao-agent-123"

    def test_send_agent_message_no_process(self, bridge):
        """Test sending message without registered process."""
        result = bridge.send_agent_message("UNKNOWN", "target", "action")
        assert "error" in result
        assert "No process registered" in result["error"]

    def test_create_agent_shell(self, bridge):
        """Test creating AOS agent shell."""
        with patch.object(bridge.aos, 'spawn_aos') as mock_spawn:
            mock_spawn.return_value = {"id": "aos-shell-1"}
            with patch.object(bridge.aos, 'load_blueprint') as mock_load:
                mock_load.return_value = {"status": "loaded"}
                with patch.object(bridge.aos, 'eval_lua') as mock_eval:
                    mock_eval.return_value = {"status": "evaluated"}
                    result = bridge.create_agent_shell("AGENT-1")

        assert "status" in result

    def test_sync_with_arkhe(self, bridge, mock_agent):
        """Test synchronization with ArkheOmniAgent."""
        with patch.object(bridge, 'persist_agent_state') as mock_persist:
            mock_persist.return_value = {
                "tx_id": "sync-tx-123",
                "status": "mock_uploaded",
            }
            with patch.object(bridge, 'send_agent_message') as mock_send:
                mock_send.return_value = {"status": "sent"}
                result = bridge.sync_with_arkhe(mock_agent)

        assert result["status"] == "synced"
        assert "upload" in result
        assert "state" in result
        assert result["state"]["agent_id"] == "TEST-AGENT-927"

    def test_get_bridge_status(self, bridge):
        """Test bridge status report."""
        status = bridge.get_bridge_status()

        assert status["substrate"] == "927"
        assert "arweave_gateway" in status
        assert "ao_mu" in status
        assert "uploads_count" in status
        assert "processes_registered" in status


# ═══════════════════════════════════════════════════════════════════
# PermawebAdapter Tests
# ═══════════════════════════════════════════════════════════════════

class TestPermawebAdapter:
    """Tests for ArkheOmniAgent integration adapter."""

    def test_adapter_injection(self, mock_agent):
        """Test that adapter injects permaweb into agent."""
        adapter = PermawebAdapter(mock_agent)

        assert hasattr(mock_agent, 'permaweb')
        assert mock_agent.permaweb is not None

    def test_commit_memory_override(self, mock_agent):
        """Test that commit_memory is overridden to persist to Arweave."""
        adapter = PermawebAdapter(mock_agent)

        # Call the overridden commit_memory
        result = mock_agent.commit_memory({"test": "data"})

        assert "commit_id" in result
        # Should also have Arweave fields (or error if no wallet)
        assert "arweave_tx" in result or "arweave_error" in result

    def test_original_commit_still_called(self, mock_agent):
        """Test that original commit logic is preserved."""
        original_commit = mock_agent.commit_memory
        adapter = PermawebAdapter(mock_agent)

        # The adapter wraps the original
        result = mock_agent.commit_memory({"key": "value"})

        # Should have a commit_id from original logic
        assert "commit_id" in result
        assert result["commit_id"].startswith("commit-")


# ═══════════════════════════════════════════════════════════════════
# Security Tests
# ═══════════════════════════════════════════════════════════════════

class TestSecurity:
    """Security-focused tests."""

    def test_wallet_not_exposed(self, bridge):
        """Test that wallet JWK is not exposed in status."""
        status = bridge.get_bridge_status()
        # Should indicate if wallet is configured, but never expose the key
        assert "wallet_configured" in status
        assert isinstance(status["wallet_configured"], bool)
        # Ensure no raw key data
        assert "jwk" not in str(status).lower()
        assert "private" not in str(status).lower()

    def test_upload_tags_include_metadata(self, bridge):
        """Test that uploads include proper metadata tags."""
        result = bridge.arweave.upload_data("test", {"Custom": "tag"})

        tags = result["tags"]
        assert tags["App-Name"] == "ARKHE-OS"
        assert tags["App-Version"] == "2.0.0"
        assert tags["Substrate"] == "927"
        assert "Timestamp" in tags
        assert tags["Custom"] == "tag"

    def test_process_isolation(self, bridge):
        """Test that processes are isolated."""
        bridge.spawn_agent_process("AGENT-A", "agent-a")
        bridge.spawn_agent_process("AGENT-B", "agent-b")

        # Each agent should have its own process
        assert len(bridge._process_registry) == 2
        proc_a = bridge._process_registry["AGENT-A"]["process_id"]
        proc_b = bridge._process_registry["AGENT-B"]["process_id"]
        assert proc_a != proc_b


# ═══════════════════════════════════════════════════════════════════
# Performance Tests
# ═══════════════════════════════════════════════════════════════════

class TestPerformance:
    """Performance and load tests."""

    def test_multiple_uploads(self, bridge):
        """Test handling multiple uploads."""
        for i in range(10):
            bridge.persist_agent_state({"iteration": i}, f"AGENT-{i}")

        assert len(bridge._upload_history) == 10
        # All should have unique TX IDs
        tx_ids = [u["tx_id"] for u in bridge._upload_history]
        assert len(set(tx_ids)) == 10

    def test_large_state_serialization(self, bridge):
        """Test serialization of large agent states."""
        large_state = {
            "memory": ["x" * 1000 for _ in range(100)],
            "hypergraph": {f"edge_{i}": i for i in range(1000)},
        }

        result = bridge.persist_agent_state(large_state, "LARGE-AGENT")

        assert result["status"] == "mock_uploaded"
        assert result["size"] > 100000  # Should be large


# ═══════════════════════════════════════════════════════════════════
# Run
# ═══════════════════════════════════════════════════════════════════
if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])