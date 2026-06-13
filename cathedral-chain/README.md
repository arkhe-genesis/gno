# Cathedral Blockchain

A Cathedral Blockchain é uma blockchain soberana projetada para executar a stack da ASI (omni-triad) sobre um ledger de consenso rápido, seguro e interoperável.
Construída sobre o Cosmos SDK, utilizando CometBFT para consenso, CosmWasm para contratos inteligentes e IBC para interoperabilidade.

## Componentes

- **Consenso**: CometBFT
- **Execução**: CosmWasm + wasmi
- **Criptografia Pós-Quântica**: SPHINCS+ (FIPS-205), ML-DSA (FIPS-204), BLS12-381
- **Governança**: DiscourseDetector + Quadratic Voting
- **Interoperabilidade**: IBC v2 + Eureka ZK
- **Aprendizado Federado**: x/federated

## Módulos

- `x/governance`: Governança com detecção de discursos.
- `x/identity`: Registro de chave pública gerada por TEE (`generateKey`).
- `x/zk`: Verificação de provas ZK (nova-snark, SNARKtor).
- `x/federated`: Aprendizado federado verificável.
