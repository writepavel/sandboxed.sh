# EIP-712 Standard: Analysis and Security Findings

## 1. Overview
EIP-712 is a standard for hashing and signing typed structured data. Before EIP-712, users often had to sign hexadecimal strings that were difficult to verify (e.g., via `eth_sign`). EIP-712 allows wallets to display data in a human-readable format, improving security and transparency.

## 2. The Hashing Process (`signTypedData_v4`)
The core of EIP-712 is the generation of a unique hash derived from:
1.  **Prefix**: `\x19\x01` (to prevent collision with other signature types).
2.  **Domain Separator Hash**: `hashStruct(domainSeparator)`
3.  **Message Hash**: `hashStruct(message)`

**Final Hash Formula:**
`keccak256("\x19\x01" ‖ domainSeparatorHash ‖ messageHash)`

### 2.1 Domain Separator
The Domain Separator prevents:
- **Cross-Chain Replays**: Uses `chainId`.
- **Cross-App Replays**: Uses `verifyingContract` (the address of the contract that will verify the signature).
- **Versioning Conflicts**: Uses `name` and `version`.

### 2.2 `hashStruct` Calculation
`hashStruct(struct) = keccak256(typeHash ‖ encodeData(struct))`

- **`typeHash`**: `keccak256(encodeType(structType))`
    - Example: `keccak256("Mail(address from,address to,string contents)")`
- **`encodeData`**: Values are 32-byte padded. Complex types (strings, dynamic arrays) are hashed separately first.

## 3. Security Findings & Common Vulnerabilities

### 3.1 Domain Separator Omission or Weakness
- **Finding**: Some implementations hardcode `chainId` or omit `verifyingContract`.
- **Impact**: Signatures meant for Mainnet might be replayable on L2s (Base, Optimism) or testnets. Omission of `verifyingContract` allows signatures to be "reused" across different protocols using the same message format.

### 3.2 Improper String and Bytes Hashing
- **Finding**: EIP-712 requires `string` and `bytes` fields to be hashed before being encoded into the struct: `encodeData(value) = keccak256(value)`.
- **Impact**: If an implementation directly uses the data without hashing, the structure becomes variable-length, leading to potential collisions or invalid signatures that fail verification in standard wallets.

### 3.3 Signature Malleability
- **Finding**: Standard `ecrecover` in Solidity is susceptible to signature malleability (using `s` values that are greater than `n/2`).
- **Mitigation**: Use OpenZeppelin’s `ECDSA` library which checks for high-S values.

### 3.4 Insecure Schema Design
- **Finding**: Including sensitive fields that can be easily manipulated or omitting nonces.
- **Impact**: Without a `nonce` or `deadline` inside the structure, a signature could be re-used multiple times within the same contract (Replay Attack).

## 4. Best Practices for Developers
1. **Dynamic ChainId**: Always fetch `block.chainid` dynamically instead of using a constant at deployment.
2. **Standard Libraries**: Use OpenZeppelin’s `EIP712` contract to handle domain separators automatically.
3. **Strict Validation**: Always check `deadline` (if applicable) and ensure `nonces` are incremented/invalidated after use.
4. **Front-end / Smart Contract Alignment**: Ensure the types in your JS/TS code exactly match the `typeHash` string in Solidity (including spaces and parameter names).
