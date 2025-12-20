# Key Endpoints for CORS Analysis - web3.bitget.com (Bitget Wallet)

## Context
Analyzed the landing pages https://web3.bitget.com/ and /en. These are marketing pages for Bitget Wallet (formerly BitKeep), a Web3 wallet supporting dApps, DeFi, etc. The site uses CSP-report-only with a broad `connect-src` directive listing allowed fetch/XHR/WS origins.

No CORS headers on landing pages (normal for static HTML).

## Identification Method
- Extracted from `connect-src` in CSP: Domains the JS is allowed to connect to.
- These are **key endpoints** as they handle sensitive operations (e.g., RPC calls for wallet balances/transactions).
- Prioritized: Blockchain RPCs (risk of data exfil if permissive CORS), custom APIs (user data).

## Key Endpoints/Domains

### 1. Blockchain RPCs (High Priority - Sensitive chain data)
| Endpoint | Purpose | CORS Policy (OPTIONS /) | Notes |
|----------|---------|--------------------------|-------|
| https://eth.llamarpc.com/ | Ethereum RPC | `Access-Control-Allow-Origin: *`<br>`Allow-Methods: GET,POST,...`<br>`Allow-Headers: content-type` | Permissive wildcard. Vulnerable to cross-origin reads. |
| https://cloudflare-eth.com/ | Ethereum RPC | `Access-Control-Allow-Origin: *`<br>`Allow-Methods: POST,OPTIONS`<br>`Allow-Headers: Content-Type` | Permissive wildcard. |
| https://api.trongrid.io/ | Tron RPC | `Access-Control-Allow-Origin: *`<br>`Allow-Methods: *`<br>`Allow-Headers: *` | Fully permissive. High risk. |
| https://sun.tronex.io/ | Tron RPC | 405 Not Allowed (no preflight) | Blocks non-simple requests. |
| https://api.shasta.trongrid.io/ | Tron Shasta Testnet | (Not tested) | Similar to mainnet. |
| https://*.bknode.vip | BitKeep nodes | (Not tested) | Custom RPCs. |

### 2. Custom Bitget/BitKeep APIs (High Priority - Potential user data)
| Endpoint | Purpose |
|----------|---------|
| https://api-web.bitkeep.asia/ | BitKeep API LB |
| https://api-web.bitkeep.biz/ | BitKeep API LB |
| https://api-web.bitkeep.life/ | BitKeep API LB |
| https://api-web.bitkeep.fun/ | BitKeep API LB |
| https://api-web.chainnear.com/ | NEAR chain API |
| https://api.nileex.io/ | Nileex (NFT/partner?) |
| https://rpc-wallet.broearn.com/ | Broearn wallet RPC |

**Recommendation**: Test these with `curl -v -H "Origin: https://attacker.com"` for reflection or wildcard. Paths likely `/api/v*/user/*`, `/wallet/*`.

### 3. WalletConnect & Others (Medium Priority)
- `*.walletconnect.org` (wss:// for sessions)
- `*.geetest.com` (captcha)
- Firebase/Google Analytics (safe, expected *)

### 4. Low Priority (CDNs/Analytics)
- static-web.jjdsn.vip, cdn.bitkeep.vip (assets)
- googleapis.com, doubleclick.net

## Risks
- **RPCs with `*`**: Malicious dApp/site can read responses (e.g., balances via `eth_call`), exfil wallet data.
- Custom APIs: If permissive, steal session/user info.
- WalletConnect: WS, harder to exploit but check.

## Next Steps
- Deobfuscate/extract JS bundles from https://static-web.jjdsn.vip/... for exact fetch URLs.
- Test full paths (e.g., `/jsonrpc` for RPCs).
- Browser automation to capture live network.

**Files**:
- Raw headers: `temp/headers_*.txt`
- CSP report: `output/csp_headers.md`
- Time: 2025-12-20 13:15 UTC