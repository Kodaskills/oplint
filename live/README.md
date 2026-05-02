# oplint-live

Online demo for [oplint](https://github.com/Kodaskills/oplint) — run Obsidian plugin linting directly from your browser, powered by a Rust + Axum server on Alpine Linux.

## Architecture

```
Browser → Debian + Axum → git clone → oplint → JSON → Browser
```

- **Frontend**: Single-page HTML with vanilla JS
- **Backend**: Rust Axum server, runs oplint as a subprocess
- **Runtime**: Alpine Linux with `gcompat` (for glibc-based oplint binary)
- **Deployment**: Render.com or Railway.app (both have free tiers)

## Quick Start

### Prerequisites
- Rust (stable)
- [just](https://github.com/casey/just#installation) (optional, for convenience commands)
- Render CLI (`render`) or Railway CLI (`railway`) for deployment
- oplint binary (optional, for local testing)

### Using `just` (Recommended)

```bash
# List all available recipes
just

# Run the server locally
just run

# Run with hot-reload (requires cargo-watch)
just watch

# Build for release
just release

# Run all checks (format + lint + test)
just check

# Deploy to Render
just render-deploy
```

### Manual Commands

```bash
# Local development
cd server && cargo run

# Build Docker image locally
docker build -t oplint-live .

# Run container locally
docker run -p 8080:8080 oplint-live
```

## Just Recipes

The `justfile` provides convenient commands for development and deployment. Run `just` to list all recipes.

| Recipe | Group | Description |
|---|---|---|
| `just` (default) | - | List all available recipes |
| `just run` | dev | Start the server locally (`cargo run`) |
| `just watch` | dev | Auto-reload on file changes (requires `cargo-watch`) |
| `just build` | build | Build the server (debug) |
| `just release` | build | Build optimized binary (`--release`) |
| `just fmt` | lint | Format code with `cargo fmt` |
| `just lint` | lint | Run clippy linter |
| `just test` | test | Run tests |
| `just check` | test | Run all checks (fmt + lint + test) |
| `just docker-build` | docker | Build Docker image locally |
| `just docker-run` | docker | Run container locally on port 8080 |
| `just render-deploy` | deploy | Deploy to Render.com |
| `just render-status` | deploy | Check Render status |
| `just railway-status` | deploy | Check Railway status |
| `just install-dev` | setup | Install dev dependencies (`cargo-watch`, `just`) |
| `just info` | info | Show project information |

### Examples

```bash
# Start development server
just run

# Run with hot-reload
just watch

# Deploy to production
just render-deploy

# Check logs after deployment
just render-status
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `PORT` | `8080` | Server listen port |

## Deployment Options

### Render.com (Recommended)

1. Create a new Web Service on [Render](https://render.com)
2. Connect your GitHub repository
3. Use the `Dockerfile` in the repo
4. Set environment variable: `PORT=8080`

Or use the CLI:
```bash
just render-deploy
```

**Free tier**: 750 hours/month, auto-sleep after 15 min

### Railway.app

Connect your GitHub repo to [Railway](https://railway.app) and deploy with the `Dockerfile`.

**Free tier**: $5 credit/month (~500 hours of usage)

## API

### POST /lint

Run oplint on a GitHub/GitLab repository.

**Request:**
```json
{
  "provider": "github",
  "owner": "user",
  "repo": "obsidian-plugin",
  "config": "optional .oplint.yaml content"
}
```

**Response:**
```json
{
  "output": "{...json lint results...}",
  "html": "<optional html report>",
  "score": 85.5,
  "grade": "B+",
  "error": null
}
```

## How It Works

1. User enters a GitHub/GitLab repo URL
2. Server clones the repo via `git clone --depth 1` to a temp directory
3. Optional `.oplint.yaml` config is written if provided
4. `oplint lint <repo-dir> -f json,html -o <report-dir>` is executed
5. JSON and HTML results are returned to the browser
6. Temporary files are cleaned up

## File Structure

```
live/
├── server/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── src/
│   │   └── main.rs              # Axum server (uses git clone)
│   └── templates/
│       └── index.html             # Demo page
├── justfile                         # Convenience commands
├── Dockerfile                       # Debian-based multi-stage build
└── README.md
```

## Notes

- **git clone**: Uses shallow clone (`--depth 1`) to minimize download size
- **Temp directories**: Unique per request using PID + timestamp to avoid conflicts
- **Fly.io**: No longer offers a free tier (as of 2025)
- **Render/Railway**: Both offer free tiers suitable for this demo
