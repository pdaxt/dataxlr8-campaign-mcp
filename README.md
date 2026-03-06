# dataxlr8-campaign-mcp

Email and outreach campaign management for DataXLR8. Create campaigns, manage recipients, run A/B tests, and track engagement metrics.

## Tools

| Tool | Description |
|------|-------------|
| create_campaign | Create a new campaign with name, type (email/linkedin/multi), target criteria, and optional start date |
| add_recipients | Add contacts to a campaign by email list or filter criteria |
| launch_campaign | Activate a campaign and begin sending. Campaign must be in draft or paused status. |
| pause_campaign | Pause an active campaign. Can be resumed with launch_campaign. |
| campaign_metrics | Get campaign metrics: opens, clicks, replies, bounces, and conversion rates |
| ab_test | Create an A/B test variant with different subject/body and recipient split percentage |
| list_campaigns | List campaigns with optional status/type filter. Supports pagination via limit/offset. |
| campaign_timeline | Get the chronological event log for a campaign. Supports pagination via limit/offset. |

## Setup

```bash
DATABASE_URL=postgres://dataxlr8:dataxlr8@localhost:5432/dataxlr8 cargo run
```

## Schema

Creates `campaigns.*` schema in PostgreSQL:
- `campaigns` - Campaign metadata (name, type, status, target criteria)
- `recipients` - Email list with delivery and engagement tracking (opens, clicks, replies)
- `variants` - A/B test variants with subject, body, and recipient allocation
- `events` - Campaign event log (sent, bounced, opened, clicked, etc)

## Part of

[DataXLR8](https://github.com/pdaxt) - AI-powered recruitment platform
