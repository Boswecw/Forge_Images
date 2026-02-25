# ForgeImages System Documentation

> BDS Documentation Protocol v1.0 — modular reference for AI-assisted development

| Part | File | Contents |
|------|------|----------|
| §1 | [01-overview-philosophy.md](01-overview-philosophy.md) | Service purpose, Six Laws, ecosystem role |
| §2 | [02-architecture.md](02-architecture.md) | Trust boundary, 3-tier model, data flow |
| §3 | [03-tech-stack.md](03-tech-stack.md) | Exact dependencies and versions (Rust + Python) |
| §4 | [04-project-structure.md](04-project-structure.md) | Directory tree, key files, module responsibilities |
| §5 | [05-config-env.md](05-config-env.md) | Environment variables, CLI flags, settings |
| §6 | [06-api-layer.md](06-api-layer.md) | Bridge HTTP endpoints, CLI subcommands, exit codes |
| §7 | [07-backend-internals.md](07-backend-internals.md) | Template system, validation pipeline, hashing, print authority, compilation |
| §8 | [08-ecosystem-integration.md](08-ecosystem-integration.md) | ForgeAgents skill, VibeForge, AuthorForge integration contracts |
| §9 | [09-error-handling.md](09-error-handling.md) | The 422 contract, exit code semantics, validation violations |
| §10 | [10-testing.md](10-testing.md) | Invariant tests, boundary tests, coverage |
| §11 | [11-handover.md](11-handover.md) | Critical constraints, deployment, future work |

## Quick Assembly

```bash
bash doc/system/BUILD.sh   # Assembles all parts into doc/fiSYSTEM.md
```

*Last updated: 2026-02-19*
