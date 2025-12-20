# EIP-712: Typed Structured Data Hashing and Signing

## Overview
EIP-712 defines a standard for hashing and signing **typed structured data** on Ethereum, improving upon raw byte-string signing (EIP-191). It allows users to sign human-readable, structured messages with clear context, enhancing usability and security for off-chain signing used on-chain (e.g., permits, meta-transactions).

**Key Goals:**
- Prevent users from signing unintended data by displaying structured info in wallets.
- Domain separation to avoid cross-contract/dApp signature replays.
- Canonical encoding compatible with Solidity structs.
- No built-in replay protection (use nonces, etc.).

## Typed Structured Data (𝕊) Format
A JSON object with four fields:

```json
{
  \"types\": {
    \"EIP712Domain\": [
      {\"name\": \"name\", \"type\": \"string\"},
      {\"name\": \"version\", \"type\": \"string\"},
      {\"name\": \"chainId\", \"type\": \"uint256\"},
      {\"name\": \"verifyingContract\", \"type\": \"address\"},
      // Additional fields OK (extensible)
    ],
    \"[CustomType]\": [ // Array of fields {name: string, type: string}
      {\"name\": \"field1\", \"type\": \"uint256\"},
      {\"name\": \"field2\", \"type\": \"MyStruct\"},
      // Supports primitives, arrays (Type[]), bytes32, string, address, tuples/structs
    ]
  },
  \"primaryType\": \"[CustomType]\", // Top-level type name
  \"domain\": { // Values for EIP712Domain fields
    \"name\": \"MyDApp\",
    \"version\": \"1\",
    \"chainId\": 1,
    \"verifyingContract\": \"0x...\"
  },
  \"message\": { // Values matching primaryType fields
    \"field1\": \"123\",
    \"field2\": { /* nested */ }
  }
}
```

**Supported Types:**
- `uint<M>`, `int<M>` (M=8..256, multiple of 8)
- `bytes<M>` (M=1..32)
- `bytes`, `string`
- `address`
- `bool`
- `array` (Type[])
- `tuple` / struct (referenced by name)

## Encoding Functions

### 1. `encodeType(primaryType, types)` → bytes
Canonical ABI-like type string:
```
TypeName(encodeType(fieldType) fieldName, ...)encodeType(fieldType)...
```
- Recurse on field types.
- No spaces.
- Arrays: `Type[]`.
- Example:
```
```
Mail(
  bytes32 topic,
  bytes32 content,
  string author,
  address recipient,
  uint256 nonce
)
string
bytes32
```

### 2. `typeHash(primaryType, types)` → bytes32
```
keccak256(encodeType(primaryType, types))
```

### 3. `encodeData(primaryType, data, types)` → bytes (ABI encoded)
- Struct: `typeHash(primaryType) || encodeData(fields...)`
- uint/int: left-padded to 32 bytes.
- bytes32/string/bytes: `keccak256(data)` if dynamic? No:
  - Fixed bytes<M>: padded.
  - Dynamic bytes/string: length (32b) || data (padded).
  - Array: length || elements (recursive).
  - Address: 32b padded.
  - Bool: 1 or 0 padded.
- **Recursive** on fields/arrays.

### 4. `hashStruct(message, 𝕊)` → bytes32
```
keccak256(encodeData(primaryType(𝕊), message, types(𝕊)))
```

### 5. `domainSeparator(𝕊)` → bytes32
```
hashStruct(domain(𝕊), 𝕊)  // Uses EIP712Domain type
```

### 6. `typedDataHash(𝕊)` → bytes32 (The signable hash!)
```
keccak256(\"\\x19\\x01\" || domainSeparator(𝕊) || hashStruct(message(𝕊), 𝕊))
```
- `\x19\x01` from EIP-191 for domain sep.

## Signing Flow
1. DApp prepares 𝕊 JSON.
2. Wallet displays structured data (name, fields).
3. User signs `typedDataHash` via `eth_signTypedData_v4` RPC.
4. On-chain: recover signer from sig, verify hash matches expected.

## RPC: `eth_signTypedData` / `eth_signTypedData_v4`
```
params: [address, typedData]  // typedData = JSON.stringify(𝕊)
```
- v1-v3 deprecated; use v4.
- Returns sig (65 bytes: r,s,v).

## Security
- **Domain Separation:** Different domains → different hashes.
- **Type Safety:** Canonical encoding prevents malleability.
- **Risks:**
  - Replay: Add nonce/expiry.
  - Frontrunning: Use commit-reveal or deadlines.
  - Wallet must display correctly.

## Example
```json
{
  \"types\": {
    \"EIP712Domain\": [...],
    \"Person\": [
      {\"name\": \"name\", \"type\": \"string\"},
      {\"name\": \"wallet\", \"type\": \"address\"}
    ],
    \"Mail\": [
      {\"name\": \"from\", \"type\": \"Person\"},
      {\"name\": \"to\", \"type\": \"Person\"},
      {\"name\": \"contents\", \"type\": \"string\"}
    ]
  },
  \"primaryType\": \"Mail\",
  \"domain\": {\"name\": \"Ether Mail\", \"version\": \"1\"},
  \"message\": {
    \"from\": {\"name\": \"Cow\", \"wallet\": \"0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826\"},
    \"to\": {\"name\": \"Bob\", \"wallet\": \"0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB\"},
    \"contents\": \"Hello, Bob!\"
  }
}
```
Hash computation yields specific bytes32.

## Implementations
- ethers.js: `TypedDataDomain`, `signTypedData`
- OpenZeppelin: `EIP712` base contract.
- Viere: Libraries for encoding.

## References
- Official Spec: https://eips.ethereum.org/EIPS/eip-712
- EIP-191 Context: https://eips.ethereum.org/EIPS/eip-191

**Status:** Final (since 2021). Widely used in DeFi (Permits in ERC-20).","path">output/eip-712-summary.md