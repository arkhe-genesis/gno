# Auditoria Formal de Segurança: CathedralGovernance.sol

## 1. Visão Geral
Este documento audita o contrato inteligente `CathedralGovernance` (Solidity, EVM compatível) projetado para a RBB Chain. O objetivo é garantir que as regras de governança da AGI sejam imutáveis e à prova de ataques comuns e Byzantinos.

## 2. Análise de Controle de Acesso e Autorização

### 2.1. Função `executeValidatedAction`
```solidity
function executeValidatedAction(bytes32 proposalId, bytes aggregatedSignature, bytes memory)
```
- **Vulnerabilidade Encontrada (Alta Severidade):** A função é `public`. Qualquer endereço pode chamá-la.
- **Impacto:** Um nó malicioso poderia invocar esta função com uma assinatura BLS forjada (se a verificação on-chain não for implementada) ou explorar o parâmetro `memory` para corromper o estado do contrato.
- **Mitigação Necessária:**
  - Adicionar `require(msg.sender == owner || hasRole(GOVERNANCE_ROLE))`.
  - Implementar verificação BLS no contrato (usando uma biblioteca pré-compilada como `bls12381-sol`) para garantir que o `aggregatedSignature` é matematicamente válida antes de atualizar o estado.

### 2.2 Mapeamento de Papéis (Role-Based Access Control - RBAC)
- **Status Atual:** Inexistente. Qualquer um com a assinatura BLS correta pode executar ações.
- **Obrigatório para RBB Chain:** A RBB Chain exige que contratos institucionais tenham controle granular. Deve ser adicionado um `mapping(address => role)` gerenciado por um multisig dos signatários originais.

## 3. Vulnerabilidades de Estado e Reentrância

### 3.1. Armazenamento de Decisões
```solidity
mapping(bytes32 => uint256) public proposalSignatures;
```
- **Análise:** Armazenamento em `mapping` em Solidity é seguro contra reentrância (não há loops), mas não fornece uma forma fácil de paginar ou consultar o histórico de forma eficiente fora da cadeia.
- **Sugestão:** Utilizar uma estrutura de Array (`Struct[] Proposal`) para permitir iteração no frontend sem incorrer em custos de gás exorbitantes.

### 3.2. Condição de Saída Vazia
```solidity
if (proposalSignatures[proposalId] >= THRESHOLD) {
    // ...
}
```
- **Análise:** Em Solidity, mapear uma chave não inicializada retorna `0`. Se o limiar `THRESHOLD` for `0`, qualquer transação acionará a lógica de aprovação por engano.
- **Mitigação:** Inicializar `proposalSignatures[proposalId] = 1` no momento da criação da proposta, ou exigir que `THRESHOLD >= 3`.

## 4. Verificação Off-Chain vs. On-Chain

O manifesto `cathedral_manifest.adp.yaml` exige que a verificação da prova Lean 4 seja on-chain.
- **Problema:** Verificar uma prova Lean 4 em EVM é computacionalmente inviável no estado da arte (Solidity não suporta os tipos de dados necessários).
- **Solução (Adotada na Cathedral): A verificação ocorre *off-chain* (pelo `safe_extraction_pipeline.py`). A blockchain armazena apenas o *hash da prova*, não a prova em si. O contrato RBB deve ser atualizado para refletir isso:
  ```solidity
  bytes32 lean4_proof_hash;
  bytes32 binary_hash;
  event ProposalExecuted(bytes32 indexed proposalId, bytes lean4_proof_hash, bytes binary_hash);
  ```

## 5. Verificação de Propriedades Desejadas

| Propriedade | Status Atual | Ajuste Necessário |
| :--- | :--- | :--- |
| **Safety (Nenhum nó age sozinho)** | Garantido por K-de-N na aplicação. | Garantido pela arquitetura, mas o contrato deve forçar que `THRESHOLD > 1`. |
| **Liveness (Decisões sempre são resolvidas) | Falha se os signatários ficarem offline. | Adicionar mecanismo de "fallback de emergência" (ex: se o quórum não for atingido em 72h, o contrato permite que um comitê de emergência aprove a proposta com `THRESHOLD = 1`). |
| **Imutabilidade (Histórico imutável)** | Logs ficam em memória volátil (off-chain). | Implementar Eventos Ethereum anexando merkle roots do estado completo da governança na RBB Chain. |

## 6. Conclusão da Auditoria
O contrato atual serve como um excelente **protótipo de conceito**. Para deployment na RBB Chain Mainnet, as 3 vulnerabilidades identificadas (Falta de RBAC, risco de limiar zero, verificação off-chain implícita) devem ser corrigidas. A arquitetura conceitual atende aos requisitos de soberania, mas a implementação Solidity precisa ser endurecida para resistir ao ambiente hostil de uma blockchain pública permissionada.
