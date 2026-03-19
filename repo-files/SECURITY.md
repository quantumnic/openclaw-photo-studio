# Security Policy

## Reporting Vulnerabilities

**Do NOT open a public issue for security vulnerabilities.**

Send reports to: **security@openclaw.photo**

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Response Timeline

| Action | Timeline |
|--------|----------|
| Acknowledgement | Within 48 hours |
| Initial assessment | Within 7 days |
| Fix development | Within 30 days (critical) / 90 days (non-critical) |
| Public disclosure | After fix is released |

## Scope

### In Scope
- Remote code execution via crafted RAW/XMP/image files
- Plugin sandbox escapes
- Catalog database injection
- Unauthorized file access via path traversal
- Memory safety issues in Rust unsafe blocks
- Information disclosure through exported files (unintended metadata)

### Out of Scope
- Bugs that require physical access to the machine
- Social engineering
- Denial of service via large files (expected behavior for RAW processing)
- Issues in third-party plugins (contact the plugin author)

## Responsible Disclosure

We follow coordinated disclosure:
1. Reporter sends vulnerability details privately
2. We confirm, assess, and develop a fix
3. We release the fix and credit the reporter (if desired)
4. Full details published after users have had time to update (typically 30 days)

## Recognition

We maintain a Security Hall of Fame in `SECURITY-THANKS.md` for responsible disclosures. No monetary bounty program at this time, but we're open to establishing one as the project grows.

## Supported Versions

| Version | Supported |
|---------|-----------|
| Latest release | ✅ |
| Previous minor | ✅ (security fixes only) |
| Older | ❌ |
