# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ Active development |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, report them privately by emailing: **security@aether-os.dev**

Include:

- Description of the vulnerability
- Steps to reproduce
- Potential impact assessment
- Suggested fix (if any)

We aim to acknowledge reports within **48 hours** and provide an initial assessment
within **7 days**.

## Scope

Security issues in the following areas are in scope:

- Kernel memory safety violations
- Privilege escalation via syscalls
- Boot chain integrity
- Supply chain (dependency vulnerabilities in build tooling)

Out of scope:

- Denial-of-service via resource exhaustion in unprivileged user code (unless it
  affects kernel stability)
- Issues in third-party tools (QEMU, OVMF) — report to upstream

## Disclosure Policy

We follow coordinated disclosure. We will work with reporters to understand and
fix issues before public disclosure. Credit will be given unless anonymity is
requested.

## Safe Harbor

We consider security research conducted in good faith to be authorized. We will
not pursue legal action against researchers who:

- Make a good faith effort to avoid privacy violations and data destruction
- Report vulnerabilities promptly
- Allow reasonable time for remediation before public disclosure
