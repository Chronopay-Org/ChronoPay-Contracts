# ChronoPay Security Assumptions

This document outlines the security assumptions, invariants, and threat model for the ChronoPay time-token contract.

## Core Assumptions

1. **Protocol Security**: We assume the underlying Soroban/Stellar ledger is secure and provides censorship resistance, data integrity, and finalized transactions.
2. **Authorized Professional**: Only a legitimate professional (e.g., a service provider) should be able to create time slots for their own availability.
3. **Token Scarcity**: Each time slot corresponds to a unique token. Once sold or redeemed, it cannot be reused for a different appointment.
4. **Environment Time**: We rely on `env.ledger().timestamp()` for time-based logic. We assume the ledger timestamp is sufficiently accurate and cannot be significantly manipulated by validators.

## Safety Invariants

- **Valid Time Slots**: `start_time` must always be strictly less than `end_time`.
- **Authorized Actions**: Actions like `mint_time_token` or `redeem_time_token` require explicit authorization from the token owner or the professional.
- **State Transitions**:
  - `Available` -> `Sold` (via purchase)
  - `Sold` -> `Redeemed` (via service fulfillment)
  - `Redeemed` is a terminal state.

## Failure-Mode Handling

- **Overflow Protection**: All arithmetic operations use checked math to prevent slot ID or balance overflows.
- **Unauthorized Access**: Missing or invalid signatures will result in `env.require_auth()` failing and the transaction reverting.
- **Invalid State**: Attempting to redeem an already redeemed token will result in a contract panic or error.

## Threat Model

### Professional Impersonation

- **Threat**: An attacker tries to create slots on behalf of a professional.
- **Mitigation**: `env.require_auth()` ensures only the caller can create slots associated with their address.

### Double Redemption

- **Threat**: A user tries to redeem the same time token multiple times to get multiple services.
- **Mitigation**: The contract state `DataKey::Status` tracks the redemption status, preventing re-redemption.
