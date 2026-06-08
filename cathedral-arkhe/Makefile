# Makefile — Cathedral ARKHE

.PHONY: .DEFAULT
SHELL := /bin/bash

# ─── Desenvolvimento ───

install:
	pip install -e ".[dev]"
	@echo "✓ Dependências de desenvolvimento instaladas"

install-llm:
	pip install -e ".[llm]"
	@echo "✓ Dependências de inferência instaladas"

install-zkml:
	pip install -e ".[zkml]"
	@echo "✓ Dependências ZKML instaladas"

install-lora:
	pip install -e ".[lora]"
	@echo "✓ Dependências LoRA instaladas"

install-all:
	pip install -e ".[all]"
	@echo "✓ Todas as dependências instaladas"

# ─── Testes ───

test:
	pytest tests/ -v --tb=short
	@echo "✓ Testes passaram"

test-fast:
	pytest tests/ -v --tb=short -k "not slow and not real"
	@echo "✓ Testes rápidos passaram"

test-coverage:
	pytest tests/ --cov=cathedral --cov-report=html -v --tb=short
	@echo "✓ Cobertura gerada em htmlcov/"

test-integration:
	pytest tests/ -v --tb=short -m integration
	@echo "✓ Testes de integração passaram"

# ─── Qualidade ───

lint:
	ruff check cathedral/
	ruff format --check cathedral/
	@echo "✓ Lint passou"

typecheck:
	mypy cathedral/ --ignore-missing-imports
	@echo "✓ Type check completou"

format:
	ruff format cathedral/
	@echo "✓ Código formatado"

# ─── Documentação ───

docs:
	python -c "from cathedral._version import SEALS; \
	           import json; \
	           print(json.dumps(SEALS, indent=2))"
	@echo "✓ Selos listados"

# ─── Demonstração ───

demo:
	python examples/full_orchestration.py
	@echo "✓ Demonstração executada"

demo-meta-attack:
	python examples/meta_attack_experiment.py
	@echo "✓ Experimento meta-attack executado"

# ─── Limpeza ───

clean:
	find . -type d -name "__pycache__" -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete
	find . -type f -name ".pytest_cache" -delete
	find reports/ -name "*.jsonl" -delete 2>/dev/null || true
	@echo "✓ Limpo"

# ─── Publicação ───

publish-test:
	python -m build
	twine check dist/*
	@echo "✓ Build e check passaram"

publish:
	python -m build
	twine upload dist/*
	@echo "✓ Publicado no PyPI"

# ─── Info ───

info:
	@echo "Cathedral ARKHE v$$(python -c 'from cathedral._version import __version__; print(__version__)')"
	@echo "Substratos: 10"
	@echo "Selos: $$(python -c 'from cathedral._version import SEALS; print(len(SEALS))')"
