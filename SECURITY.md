# Security Policy

Fluid Island is not yet a released application. Security reporting procedures will be expanded before stable releases.

## Reporting a vulnerability

Please do not publish exploitable details in a public issue. Use GitHub's private vulnerability reporting feature if it is enabled for this repository. Otherwise, contact the maintainers privately through the contact information on the project profile.

Include the affected revision, environment, reproduction steps, impact, and any practical mitigation. Do not include passwords, tokens, or unrelated personal data.

## Scope principles

Security reviews should consider process permissions, IPC boundaries, shell integration, plugin loading, external commands, filesystem access, and any network activity. The application should request only the permissions it needs.
