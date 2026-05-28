import pytest
from visual_ontology_engine import VisualOntologyEngine

def test_ontology_ingest_and_export():
    engine = VisualOntologyEngine(ontology_path="examples/gno.land/p/arkhe/fhe_artifacts/schema_943.jsonld")
    ds = {
        "colors": {"test_color": "#FFFFFF"},
        "typography": {"test_font": "Arial"},
        "components": [{"name": "Btn"}]
    }
    res = engine.ingest_design_system(ds)
    assert res["tokens_ingested"] == 2
    assert res["components_ingested"] == 1

    winui_export = engine.export("winui3", "xaml")
    assert "test_color" in winui_export
    assert "test_font" in winui_export
