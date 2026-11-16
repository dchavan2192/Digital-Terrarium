# Project-only unattended access: prepared template, not active

The owner requested access to this project, GitHub and the internet without
repeated approvals, while keeping other personal directories inaccessible.
[CODEX_PROJECT_PERMISSIONS.toml](CODEX_PROJECT_PERMISSIONS.toml) is a proposed
local-command permission profile for a compatible Codex host. It has not been
installed or validated as an effective sandbox policy. This conversation uses
a host-managed policy, which an agent cannot replace through project files.

The template denies filesystem access by default, grants this exact project
read/write access, permits its Git metadata, and protects hooks and agent config.
It permits public command-network access and the already authorized Pi address.
It denies general system-temp access. `:minimal` retains system/runtime reads
needed to execute tools, so this is not literally zero access to every directory
outside the project. Additional installed toolchains may be blocked; do not
silently expand the filesystem allowlist if that happens.

`approval_policy="never"` prevents approval questions; it does **not** turn
blocked operations into permitted ones. Managed requirements and separate
connector/browser/Computer Use permissions still apply. Permission profiles do
not compose with legacy `sandbox_mode`/`sandbox_workspace_write` configuration;
higher-priority legacy or managed settings can prevent this template taking
effect. Work Cloud/local-computer access has its own policy controls, so do not
assume this local Codex example applies to it.

## Activation and verification

In the host's permission configuration, select the equivalent custom policy or
load this template through its supported configuration mechanism, then start a
new session. Do not replace an existing global config wholesale. Remove only
conflicting legacy settings when adopting permission profiles, and preserve
unrelated account/model settings. A host administrator may need to allow this
profile. No global configuration was read or changed for this preparation.

Before treating it as enforced, verify in the new session that a project test
file can be read/written, an unrelated user directory is denied, public HTTPS
works, and ordinary repository Git writes work. Check effective policy; TOML
syntax validation alone is not a sandbox test. Avoid Full Access, which does
not supply the requested directory boundary.

Use a project-local temporary directory for tools that require one. GitHub or
SSH commands may require credentials outside the project; supply them through
the host's credential integration or process environment rather than granting
all of the home directory or copying secrets into the repository. Network
access to an SSH server is not a remote-filesystem sandbox: Pi commands remain
scoped operationally to `/home/pi/Digital-Terrarium`, but strict remote isolation
requires server-side enforcement.

Official references checked on 2026-11-16:

- [Permission profiles and workspace-only example](https://learn.chatgpt.com/docs/permissions)
- [Approval policies and Work Cloud/local execution](https://learn.chatgpt.com/docs/agent-approvals-security)
- [Configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference)
