import pytest
import json
from unittest.mock import patch
from harness_adapter import ArkheMCPServer

def test_harness_initialize():
    server = ArkheMCPServer()
    res = server.handle_request({"method": "initialize", "id": 1})
    assert res["result"]["protocolVersion"] == "1.0"
    assert "invoke_glasswing" in res["result"]["capabilities"]["tools"]["list"]

def test_harness_tool_call():
    server = ArkheMCPServer()
    res = server.handle_request({
        "method": "tools/call",
        "params": {
            "name": "set_cognitive_effort",
            "arguments": {"level": "high"}
        },
        "id": 2
    })
    assert res["result"]["result"]["level_set"] == "high"
