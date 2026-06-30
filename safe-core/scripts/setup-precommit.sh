#!/bin/bash
pip3 install --user pre-commit
pre-commit install
cargo install cargo-audit cargo-deny cargo-tarpaulin
