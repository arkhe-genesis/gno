# Taproot Assets Authentication Guide

Authentication with the `tapd` daemon requires two components:
1. **TLS/SSL Connection**: Encrypted communication.
2. **Macaroons**: Scoped authentication tokens.

To connect, the client uses `ClientTlsConfig` and passes the Macaroon hex-encoded in the `macaroon` metadata header for every request.
