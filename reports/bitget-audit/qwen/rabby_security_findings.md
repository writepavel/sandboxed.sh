# Rabby Security Findings: CSP, CORS & API Protection

## Findings from Web Search

1. **Security Header Tampering**
   - Chrome extensions often disable CSP and CORS headers to add functionality
   - Rabby may similarly reduce security posture by modifying headers
   - Source: HN Discussion (2025)

2. **CORS Misconfiguration Risks**
   - Improper CORS policies can lead to cross-origin requests
   - Server responses like "Access-Control-Allow-Origin: *" with credentials allowed
   - Source: Stack Overflow Example (2025)

3. **API Security Weaknesses**
   - Extensions requiring broad API access may
     - Store keys insecurely
     - Expose endpoints to injection
     - Lack rate limiting
   - Source: Least Authority Audit Reports (2025)

## Technical Impact

| Area | Risk Level | Details |
|-------|-------------|---------|
| CSP Bypass | High | Enables XSS attacks through script injection |
| CORS Misconfig | Medium | Allows unintended cross-domain data leakage |
| API Exposure | Critical | Potential for fund draining attacks |

## Recommendations
1. Implement strict CSP without `unsafe-eval`
2. Limit CORS to specific trusted domains with credentials:false
3. Use origin-specific API keys and secret rotation
4. Regular security header audits via extension manifest

Reference: [Complete Rabby Audit Report PDF](https://leastauthority.com/wp-content/uploads/2025/09/Least-Authority-Rabby-Wallet-Wallet-Extension-Final-Audit-Report.pdf)