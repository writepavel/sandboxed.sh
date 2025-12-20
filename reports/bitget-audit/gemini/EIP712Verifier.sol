// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import "@openzeppelin/contracts/utils/cryptography/EIP712.sol";

contract EIP712Verifier is EIP712 {
    using ECDSA for bytes32;

    // Type hashes for structured data components
    bytes32 private constant PERSON_TYPEHASH = keccak256("Person(string name,address wallet)");
    bytes32 private constant MAIL_TYPEHASH = keccak256("Mail(Person from,Person to,string contents)Person(string name,address wallet)");

    struct Person {
        string name;
        address wallet;
    }

    struct Mail {
        Person from;
        Person to;
        string contents;
    }

    constructor(string memory name, string memory version) EIP712(name, version) {}

    function hashPerson(Person memory person) private pure returns (bytes32) {
        return keccak256(abi.encode(
            PERSON_TYPEHASH,
            keccak256(bytes(person.name)),
            person.wallet
        ));
    }

    function hashMail(Mail memory mail) private pure returns (bytes32) {
        return keccak256(abi.encode(
            MAIL_TYPEHASH,
            hashPerson(mail.from),
            hashPerson(mail.to),
            keccak256(bytes(mail.contents))
        ));
    }

    function verify(Mail memory mail, bytes memory signature) public view returns (address) {
        bytes32 digest = _hashTypedDataV4(hashMail(mail));
        return digest.recover(signature);
    }
    
    function getDomainSeparator() public view returns (bytes32) {
        return _domainSeparatorV4();
    }
}
