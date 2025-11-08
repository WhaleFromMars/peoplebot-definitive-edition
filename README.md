## Releases

Publishing a GitHub release builds the pinned Docker image via `.github/workflows/release.yml` and pushes it to `ghcr.io/whalefrommars/peoplebot`.

## Required envs

Create a `.env` (or use `.env.example` as a template) with the following variables before running the bot locally or in production.
All variables must be prefixed with DEV_, PROD_, or BOTH_, depending on where you want that env to be available.
Debug builds will check for DEV_ prefixed ENVs then fallback to BOTH_ if not present.
Release builds will check for PROD_ prefixed ENVs then fallback to BOTH_ if not present.
Non prefixed ENVs will not be used.

- `PROD_DISCORD_TOKEN` / `DEV_DISCORD_TOKEN` – Discord bot tokens for the production and development apps.
- `DEV_GUILD_ID` – Guild ID used for fast slash-command registration during development (18-digit server ID).
- `BOTH_EMBEDDER_SIZE_LIMIT` – Maximum number of bytes the embedder is allowed to download when enabled.
- `BOTH_EMBEDDER_CONCURRENCY_LIMIT` – Concurrent download limit for the embedder module.
