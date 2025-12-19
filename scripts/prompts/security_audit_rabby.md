# Security Audit Task

## YOUR WORKING FOLDER (MANDATORY)
**ALL files you create MUST go in: `/root/work/security-audit-{your-model-name}/`**

Create this structure immediately:
```
/root/work/security-audit-{model}/
├── output/
│   └── AUDIT_REPORT.md    # Your final deliverable (REQUIRED)
├── temp/                   # Working files, downloads, extractions
└── notes.md               # Your analysis notes and findings
```

## TARGET
**Rabby Wallet Chrome Extension** - A cryptocurrency wallet with transaction simulation.

Source options (in order of preference):
1. Pre-downloaded in `/root/context/` (check first with `ls -la /root/context/`)
2. Chrome Web Store direct download: `curl -L "https://clients2.google.com/service/update2/crx?response=redirect&x=id%3Dacmacodkjbdgmoleebolmdjonilkdbch%26uc" -o rabby.crx`
3. GitHub source: `git clone https://github.com/RabbyHub/Rabby`

## SCOPE - FOCUS ONLY ON THESE AREAS
1. **Transaction Simulation Bypass** - Can attackers make harmful transactions appear safe?
2. **Approval Amount Manipulation** - Can displayed approval amounts differ from actual?
3. **Spender Address Spoofing** - Can fake addresses be shown as trusted protocols?
4. **Permit2 Integration** - Validation of spender field against known reactors/protocols

## REFERENCE VULNERABILITY (Example of what to find)
A previous critical bug was found where Permit2 transactions could bypass simulation:
- **Symptom**: Simulation showed "Spend 1 USDC to receive 1337 ETH"
- **Reality**: Transaction approved 100,000 USDC to attacker's vanity address
- **Root cause**: The `spender` field in Permit2 was not validated against trusted addresses
- **Why it worked**: Rabby trusted the `witness` data for simulation, but the witness can only be trusted if the spender is a known protocol (like Uniswap's reactor)
- **Impact**: Full balance drain of any approved token

Your goal is to find similar issues where trust assumptions allow bypassing security checks.

## KEY FILES TO ANALYZE
When you extract the extension, focus on:
- `background.js` - Main extension logic, message handling
- Files containing: `Permit2`, `signTypedData`, `eth_sendTransaction`, `securityEngine`
- Transaction preview/simulation components
- Approval handling and display logic

## METHODOLOGY
1. **Setup** (max 10 tool calls):
   - Create your working folder
   - Check `/root/context/` for existing files
   - Download/extract extension if needed
   
2. **Analysis** (main work):
   - Index files with `index_files` tool
   - Search for Permit2, approval, simulation code
   - Trace data flow from user input to display
   - Identify trust boundaries and validation gaps

3. **Findings**:
   - Document each issue with location, description, impact
   - Propose proof-of-concept approaches
   - Rate severity: CRITICAL / HIGH / MEDIUM / LOW

## DELIVERABLE (REQUIRED)
Your FINAL message must contain the complete `AUDIT_REPORT.md` in markdown format.

Structure:
```markdown
# Rabby Wallet Security Audit Report

**Auditor**: [your model name]
**Date**: [today's date]
**Scope**: Transaction simulation, Permit2, Approval handling

## Executive Summary
[2-3 sentences on overall security posture and key findings]

## Critical Findings

### [SEVERITY] Finding Title
- **Location**: `path/to/file.js` (line X if known)
- **Description**: Technical explanation of the vulnerability
- **Attack Scenario**: How an attacker could exploit this
- **Impact**: What damage could result (token theft, approval hijack, etc.)
- **PoC Concept**: Steps to reproduce or demonstrate
- **Recommendation**: How to fix

[Repeat for each finding]

## Medium/Low Findings
[Same format, grouped by severity]

## Code Quality Observations
[Any concerning patterns, missing validations, etc.]

## Files Analyzed
| File | Purpose | Notes |
|------|---------|-------|
| background.js | Main logic | Contains Permit2 handling |
| ... | ... | ... |

## Methodology
- Tools used: [list]
- Time spent: [estimate]
- Approach: [brief description]

## Conclusion
[Summary and actionable recommendations]
```

## STRICT RULES
1. ❌ **DON'T** create files outside `/root/work/security-audit-{model}/`
2. ❌ **DON'T** analyze unrelated files (Vulcan.jar, other extensions)
3. ❌ **DON'T** stop without producing the AUDIT_REPORT.md content
4. ✅ **DO** include the full report in your final message (not just a file path)
5. ✅ **DO** call `complete_mission` when finished with a summary
6. ✅ **DO** save the report to `/root/work/security-audit-{model}/output/AUDIT_REPORT.md`
