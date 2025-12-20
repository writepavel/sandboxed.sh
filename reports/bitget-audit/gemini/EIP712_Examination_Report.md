# EIP-712 Examination: Signing and Verification

## 1. Overview
EIP-712 is a standard for hashing and signing typed structured data. It improves security by making signed data human-readable and preventing replay attacks through domain separation.

## 2. Theoretical Framework

### Components of EIP-712
1.  **Domain Separator (`domainSeparator`)**: A hash that unique identifies the DApp/Contract context (Name, Version, ChainID, VerifyingContract).
2.  **Type Hash (`typeHash`)**: The keccak256 hash of the type signature (e.g., `Mail(Person from,Person to,string contents)`).
3.  **Typed Data Hash (`hashStruct`)**: Recursive hashing of each field according to its type.
4.  **Final Digest**: `keccak256("\x19\x01" ‖ domainSeparator ‖ hashStruct(message))`

### The Prefix `\x19\x01`
- `\x19`: Ethereum's signature prefix byte (EIP-191).
- `\x01`: Version byte specific to EIP-712 (structured data).

## 3. Implementation Analysis

### Signing (Off-chain)
In Python (using `eth-account`), the process involves defining the JSON-like structure and using `encode_typed_data`:
```python
structured_data = {
    "types": { ... },
    "primaryType": "Mail",
    "domain": { ... },
    "message": { ... }
}
signable_msg = encode_typed_data(full_message=structured_data)
signed_message = account.sign_message(signable_msg)
```

### Verification (On-chain)
In Solidity, the `EIP712` contract from OpenZeppelin handles the domain separator automatically. Developers must provide the `typeHash` and logic to hash the message struct.
The core verification step:
```solidity
bytes32 digest = _hashTypedDataV4(keccak256(abi.encode(TYPE_HASH, ...fields)));
address signer = ECDSA.recover(digest, signature);
```

## 4. Test Results Summary
A demonstration script was executed to verify the end-to-end flow.

| Property | Value |
| :--- | :--- |
| Signer Address | 0x19E7E376E7C213B7E7e7e46cc70A5dD086DAff2A |
| Signature | f1d661...1b |
| Result | Successfully recovered and verified |

*Detailed results are available in `output/eip712_test_results.json`.*

## 5. Security Considerations
- **Replay Protection**: Always include `chainId` and `verifyingContract` in the domain separator.
- **Ambiguity**: Ensure complex nested types follow the alphabetical ordering rule for type composition in the string signature.
- **Forwarding**: Be cautious of "signature malleability" if using low-level ecrecover without library protections.
