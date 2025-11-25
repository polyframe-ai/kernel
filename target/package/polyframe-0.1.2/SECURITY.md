# Security Policy

## Reporting Security Issues

Please report security issues privately to **security@polyframe.ai**.

We will acknowledge your report within **72 hours** and work with you to understand and address the issue.

### What to Include

When reporting a security vulnerability, please include:

- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Any suggested fixes (if available)

### What NOT to Do

- **Do not** open public GitHub issues for security vulnerabilities
- **Do not** disclose the vulnerability publicly until we've had a chance to address it

## Response Timeline

1. **Initial Response**: Within 72 hours of report
2. **Assessment**: We will assess the severity and impact within 1 week
3. **Fix Development**: Timeline depends on severity (critical issues prioritized)
4. **Disclosure**: Coordinated disclosure after fix is available

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Security Best Practices

When using Polyframe Kernel:

- Always validate and sanitize user-provided `.scad` files before processing
- Be aware of resource exhaustion attacks (complex geometries, deep recursion)
- Use sandboxing when processing untrusted input
- Keep dependencies up to date

## Security Updates

Security patches will be released as soon as possible and announced via:

- GitHub Security Advisories
- Release notes
- Project README

Thank you for helping keep Polyframe Kernel secure!

