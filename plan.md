1.  **Investigate the structure of the existing code.**
    -  I found that the user has provided the scripts to generate the code for both `Hashtree Bridge (Substrate 1101)` and `OnChainCanonizer (Substrate 1100)`.
    -  I successfully generated the `Hashtree Bridge (Substrate 1101)` under `examples/gno.land/p/arkhe/fhe_artifacts/`. I also modified the `substrate_registry.yaml` and `schema_1101.yaml` files.
    -  I successfully generated the `OnChainCanonizer (Substrate 1100)` under `cathedral-arkhe/cathedral/substrates/onchain/`. I fixed a bug inside `cathedral-arkhe/cathedral/substrates/onchain/proof_chain.py` (changed `self.ordered_indices.append(node.index)` to `self.ordered_indices.append(node.proof_hash)`) which was causing the tests to fail. I added it to the orchestrator pipeline, and updated the `cathedral-arkhe/cathedral/_version.py` file with the correct seals. I verified all tests are passing.

2.  **Pre-commit checks.**
    -  Use `pre_commit_instructions` tool to get instructions on checks before submit, and execute them.

3.  **Submit the changes.**
    -  Use `submit` tool to finalize my work.
