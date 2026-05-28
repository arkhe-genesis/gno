import pytest
from catedral_code_agent import CatedralCodeAgent

def test_agent_init():
    agent = CatedralCodeAgent()
    session = agent.init_session(".", "python", "surface")
    assert session.language == "python"
    assert session.audit_depth == "surface"
    assert "944" in session.active_substrates

def test_agent_audit():
    agent = CatedralCodeAgent()
    session = agent.init_session(".")
    res = agent.audit(session.session_id)
    assert res["status"] == "completed"
