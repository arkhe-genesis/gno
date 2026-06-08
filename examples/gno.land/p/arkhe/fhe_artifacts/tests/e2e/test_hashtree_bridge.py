import pytest
from hashtree_bridge import HashtreeBridge1101, HashtreeVisibility

def test_hashtree_bridge_persist_memory_lake():
    bridge = HashtreeBridge1101(
        nostr_private_key="demo_key_12345",
        visibility=HashtreeVisibility.LINK_VISIBLE,
    )
    lake_entries = [
        {"entry_hash": "0xabc...", "type": "KERNEL_INTEGRITY", "data": {}},
        {"entry_hash": "0xdef...", "type": "GARAK_SCAN_RESULT", "data": {}},
    ]
    cid = bridge.persist_memory_lake(lake_entries, encrypt=True)
    assert cid is not None
    assert str(cid).startswith("nhash:")
    assert bridge.merkle.get_telemetry()["total_nodes"] == 1
    assert bridge.nostr.get_telemetry()["events_published"] == 1

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
