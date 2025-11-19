## PeopleBot - Definitive Edition
Currently builds against:
- ffmpeg - 8.0
- yt_dlp - 2025.11.12
- Deno - v2.5.6

## Releases

Docker Images are built for this repo via `.github/workflows/release.yml` which pushes the image to `ghcr.io/whalefrommars/peoplebot`. \
Current image tags are:
- :main         – latest build from the main branch
- :latest       – latest published release
- :vX.Y.Z       – specific release version (e.g. :v1.2.3)
- :short-sha    – image built from a specific main commit (7-char git SHA, e.g. :3f4a9c2)

only the last 30 main commits are kept for :short-sha

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

## Roadmap

- [ ] Wrap ffmpeg directly instead of using yt_dlps post processing, for better control
- [ ] use a database for storing user preferences (default command flags) and guild specific settings/envs
  - [ ] planned guild settings:
    - [ ] prefix
    - [ ] language
    - [ ] toggle webm usage for embedder module (defaults to disabled atm)
    - [ ] s3 url/auth key
    - [ ] preferred quality of embeds (defaults to 1440p)
- [ ] s3 upload functionality for embedder module when the file size exceeds the guilds upload limit (each guild can specify their s3 instance/auth with the above)
