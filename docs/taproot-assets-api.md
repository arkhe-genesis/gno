# Taproot Assets API Mapping

This document provides a mapping of the Taproot Assets gRPC API.

## TaprootAssets Service (Core)
* `GetInfo` - Returns info about the daemon
* `ListAssets` - Lists assets in the daemon
* `ListBalances` - Lists balances of assets
* `ListGroups` - Lists known asset groups
* `ListTransfers` - Lists tracked transfers
* `ListUtxos` - Lists managed UTXOs
* `NewAddr` - Creates a new address for receiving
* `QueryAddrs` - Queries stored addresses
* `SendAsset` - Sends assets to an address
* `BurnAsset` - Burns units of an asset
* `DecodeAddr` - Decodes a Taproot Asset address
* `VerifyProof` - Verifies an asset proof
* `ExportProof` - Exports an asset proof
* `DecodeProof` - Decodes a proof into readable format
* `FetchAssetMeta` - Fetches asset metadata

## AssetWallet Service
* `CreateAsset` (Mapped to `MintAsset`) - Creates a new asset
* `IssueAsset` - Issues additional units

## Universe Service
* `QueryUniverse` - Queries proofs in the Universe Federation
