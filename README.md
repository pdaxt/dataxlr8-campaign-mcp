# :mega: dataxlr8-campaign-mcp

Campaign management for AI agents — create campaigns, manage recipients, A/B test, and track engagement metrics.

[![Rust](https://img.shields.io/badge/Rust-2024_edition-orange?logo=rust)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-rmcp_0.17-blue)](https://modelcontextprotocol.io/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

## What It Does

Runs outreach campaigns across email, LinkedIn, and multi-channel from MCP tool calls. Create campaigns with target criteria, add recipients by list or filter, launch and pause at will, run A/B test variants with split percentages, and track opens, clicks, replies, and conversions — all persisted in PostgreSQL with full event timelines.

## Architecture

```
                    ┌─────────────────────────┐
AI Agent ──stdio──▶ │  dataxlr8-campaign-mcp  │
                    │  (rmcp 0.17 server)      │
                    └──────────┬──────────────┘
                               │ sqlx 0.8
                               ▼
                    ┌─────────────────────────┐
                    │  PostgreSQL              │
                    │  schema: campaigns       │
                    │  ├── campaigns           │
                    │  ├── recipients          │
                    │  ├── variants            │
                    │  └── events              │
                    └─────────────────────────┘
```

## Tools

| Tool | Description |
|------|-------------|
| `create_campaign` | Create a campaign with name, type (email/linkedin/multi), and target criteria |
| `add_recipients` | Add contacts by email list or filter criteria |
| `launch_campaign` | Activate a draft or paused campaign |
| `pause_campaign` | Pause an active campaign |
| `campaign_metrics` | Get opens, clicks, replies, bounces, and conversion rates |
| `ab_test` | Create an A/B variant with different subject/body and split percentage |
| `list_campaigns` | List campaigns with optional status and type filters |
| `campaign_timeline` | Get the chronological event log for a campaign |

## Quick Start

```bash
git clone https://github.com/pdaxt/dataxlr8-campaign-mcp
cd dataxlr8-campaign-mcp
cargo build --release

export DATABASE_URL=postgres://user:pass@localhost:5432/dataxlr8
./target/release/dataxlr8-campaign-mcp
```

The server auto-creates the `campaigns` schema and all tables on first run.

## Configuration

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `LOG_LEVEL` | No | Tracing level (default: `info`) |

## Claude Desktop Integration

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "dataxlr8-campaign": {
      "command": "./target/release/dataxlr8-campaign-mcp",
      "env": {
        "DATABASE_URL": "postgres://user:pass@localhost:5432/dataxlr8"
      }
    }
  }
}
```

## Part of DataXLR8

One of 14 Rust MCP servers that form the [DataXLR8](https://github.com/pdaxt) platform — a modular, AI-native business operations suite. Each server owns a single domain, shares a PostgreSQL instance, and communicates over the Model Context Protocol.

## License

MIT
