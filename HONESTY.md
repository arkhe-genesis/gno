# HONESTY.md - Declarações de Transparência Ética

Este documento serve para registrar quaisquer desvios da implementação especificada na Constituição Viva da Cathedral ARKHE, cumprindo a **Cláusula da Transparência Ética**.

> *Qualquer desvio da implementação especificada (ex: simulação de QRNG, TEE em modo software) deve ser explicitamente declarado no ficheiro HONESTY.md na raiz do repositório. A omissão constitui violação grave da Constituição Viva.*

## Desvios Conhecidos e Declarações

*Atualmente, não existem desvios a declarar para a mainnet. Simulações em ambiente local (ex: TEE mock, /dev/urandom no lugar de QRNG verdadeiro) devem ser documentadas abaixo antes de participar do consenso.*

### Ambiente de Desenvolvimento / Testes
- A geração de seeds em testes unitários ou ambientes de desenvolvimento locais pode utilizar um PRNG convencional (ex: `os.urandom` do Python ou `RAND_bytes` do OpenSSL em modo software), pois o acesso a hardware TEE / QRNG não é garantido ou aplicável em integração contínua (CI).
- O Orchestrator em modo `--simulate` ou durante testes E2E não irá interagir com o enclave TEE real para gerar a identidade.
