use anyhow::Result;
use sqlx::PgPool;

pub async fn setup_schema(pool: &PgPool) -> Result<()> {
    sqlx::raw_sql(
        r#"
        CREATE SCHEMA IF NOT EXISTS campaigns;

        CREATE TABLE IF NOT EXISTS campaigns.campaigns (
            id               TEXT PRIMARY KEY,
            name             TEXT NOT NULL,
            campaign_type    TEXT NOT NULL DEFAULT 'email'
                             CHECK (campaign_type IN ('email', 'linkedin', 'multi')),
            target_criteria  JSONB NOT NULL DEFAULT '{}',
            status           TEXT NOT NULL DEFAULT 'draft'
                             CHECK (status IN ('draft', 'active', 'paused', 'completed', 'cancelled')),
            started_at       TIMESTAMPTZ,
            created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
        );

        CREATE TABLE IF NOT EXISTS campaigns.recipients (
            id               TEXT PRIMARY KEY,
            campaign_id      TEXT NOT NULL REFERENCES campaigns.campaigns(id) ON DELETE CASCADE,
            contact_email    TEXT NOT NULL,
            variant_id       TEXT,
            status           TEXT NOT NULL DEFAULT 'pending'
                             CHECK (status IN ('pending', 'sent', 'bounced', 'failed')),
            sent_at          TIMESTAMPTZ,
            opened_at        TIMESTAMPTZ,
            clicked_at       TIMESTAMPTZ,
            replied_at       TIMESTAMPTZ,
            UNIQUE (campaign_id, contact_email)
        );

        CREATE TABLE IF NOT EXISTS campaigns.variants (
            id               TEXT PRIMARY KEY,
            campaign_id      TEXT NOT NULL REFERENCES campaigns.campaigns(id) ON DELETE CASCADE,
            name             TEXT NOT NULL,
            subject          TEXT NOT NULL,
            body             TEXT NOT NULL,
            recipient_pct    DOUBLE PRECISION NOT NULL DEFAULT 100.0,
            created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
        );

        CREATE TABLE IF NOT EXISTS campaigns.events (
            id               TEXT PRIMARY KEY,
            campaign_id      TEXT NOT NULL REFERENCES campaigns.campaigns(id) ON DELETE CASCADE,
            event_type       TEXT NOT NULL,
            detail           TEXT,
            created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
        );

        CREATE INDEX IF NOT EXISTS idx_campaigns_status ON campaigns.campaigns(status);
        CREATE INDEX IF NOT EXISTS idx_campaigns_type ON campaigns.campaigns(campaign_type);
        CREATE INDEX IF NOT EXISTS idx_recipients_campaign ON campaigns.recipients(campaign_id);
        CREATE INDEX IF NOT EXISTS idx_recipients_email ON campaigns.recipients(contact_email);
        CREATE INDEX IF NOT EXISTS idx_variants_campaign ON campaigns.variants(campaign_id);
        CREATE INDEX IF NOT EXISTS idx_events_campaign ON campaigns.events(campaign_id);
        CREATE INDEX IF NOT EXISTS idx_events_created ON campaigns.events(created_at);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
