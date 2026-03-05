use dataxlr8_mcp_core::mcp::{empty_schema, error_result, get_f64, get_str, get_str_array, json_result, make_schema};
use dataxlr8_mcp_core::Database;
use rmcp::model::*;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ServerHandler;
use serde::{Deserialize, Serialize};
use tracing::info;

// ============================================================================
// Data types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Campaign {
    pub id: String,
    pub name: String,
    pub campaign_type: String,
    pub target_criteria: serde_json::Value,
    pub status: String,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Recipient {
    pub id: String,
    pub campaign_id: String,
    pub contact_email: String,
    pub variant_id: Option<String>,
    pub status: String,
    pub sent_at: Option<chrono::DateTime<chrono::Utc>>,
    pub opened_at: Option<chrono::DateTime<chrono::Utc>>,
    pub clicked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub replied_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Variant {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub subject: String,
    pub body: String,
    pub recipient_pct: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CampaignEvent {
    pub id: String,
    pub campaign_id: String,
    pub event_type: String,
    pub detail: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct CampaignMetrics {
    pub campaign_id: String,
    pub campaign_name: String,
    pub total_recipients: i64,
    pub sent: i64,
    pub opened: i64,
    pub clicked: i64,
    pub replied: i64,
    pub bounced: i64,
    pub open_rate: f64,
    pub click_rate: f64,
    pub reply_rate: f64,
    pub bounce_rate: f64,
}

// ============================================================================
// Tool definitions
// ============================================================================

fn build_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "create_campaign".into(),
            title: None,
            description: Some(
                "Create a new campaign with name, type (email/linkedin/multi), target criteria, and optional start date"
                    .into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "name": { "type": "string", "description": "Campaign name" },
                    "type": { "type": "string", "enum": ["email", "linkedin", "multi"], "description": "Campaign type (default: email)" },
                    "target_criteria": { "type": "object", "description": "Target criteria as JSON (e.g. industry, role, location filters)" },
                    "start_date": { "type": "string", "description": "Scheduled start date (ISO 8601). Omit for manual launch." }
                }),
                vec!["name"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "add_recipients".into(),
            title: None,
            description: Some(
                "Add contacts to a campaign by email list or filter criteria".into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "campaign_id": { "type": "string", "description": "Campaign ID" },
                    "emails": { "type": "array", "items": { "type": "string" }, "description": "List of contact emails to add" }
                }),
                vec!["campaign_id", "emails"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "launch_campaign".into(),
            title: None,
            description: Some(
                "Activate a campaign and begin sending. Campaign must be in draft or paused status.".into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "campaign_id": { "type": "string", "description": "Campaign ID to launch" }
                }),
                vec!["campaign_id"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "pause_campaign".into(),
            title: None,
            description: Some(
                "Pause an active campaign. Can be resumed with launch_campaign.".into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "campaign_id": { "type": "string", "description": "Campaign ID to pause" }
                }),
                vec!["campaign_id"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "campaign_metrics".into(),
            title: None,
            description: Some(
                "Get campaign metrics: opens, clicks, replies, bounces, and conversion rates".into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "campaign_id": { "type": "string", "description": "Campaign ID" }
                }),
                vec!["campaign_id"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "ab_test".into(),
            title: None,
            description: Some(
                "Create an A/B test variant with different subject/body and recipient split percentage".into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "campaign_id": { "type": "string", "description": "Campaign ID" },
                    "name": { "type": "string", "description": "Variant name (e.g. 'Variant A', 'Control')" },
                    "subject": { "type": "string", "description": "Email subject line for this variant" },
                    "body": { "type": "string", "description": "Email body for this variant" },
                    "recipient_pct": { "type": "number", "description": "Percentage of recipients for this variant (0-100, default 50)" }
                }),
                vec!["campaign_id", "name", "subject", "body"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "list_campaigns".into(),
            title: None,
            description: Some(
                "List all campaigns with status. Optional filter by status or type.".into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "status": { "type": "string", "enum": ["draft", "active", "paused", "completed", "cancelled"], "description": "Filter by status" },
                    "type": { "type": "string", "enum": ["email", "linkedin", "multi"], "description": "Filter by campaign type" },
                    "limit": { "type": "integer", "description": "Max results (default 50)" }
                }),
                vec![],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        Tool {
            name: "campaign_timeline".into(),
            title: None,
            description: Some(
                "Get the chronological event log for a campaign".into(),
            ),
            input_schema: make_schema(
                serde_json::json!({
                    "campaign_id": { "type": "string", "description": "Campaign ID" },
                    "limit": { "type": "integer", "description": "Max events to return (default 100)" }
                }),
                vec!["campaign_id"],
            ),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
    ]
}

// ============================================================================
// MCP Server
// ============================================================================

#[derive(Clone)]
pub struct CampaignMcpServer {
    db: Database,
}

impl CampaignMcpServer {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn get_i64(args: &serde_json::Value, key: &str) -> Option<i64> {
        args.get(key).and_then(|v| v.as_i64())
    }

    async fn log_event(&self, campaign_id: &str, event_type: &str, detail: Option<&str>) {
        let id = uuid::Uuid::new_v4().to_string();
        let _ = sqlx::query(
            "INSERT INTO campaigns.events (id, campaign_id, event_type, detail) VALUES ($1, $2, $3, $4)",
        )
        .bind(&id)
        .bind(campaign_id)
        .bind(event_type)
        .bind(detail)
        .execute(self.db.pool())
        .await;
    }

    // ---- Tool handlers ----

    async fn handle_create_campaign(&self, args: &serde_json::Value) -> CallToolResult {
        let name = match get_str(args, "name") {
            Some(n) => n,
            None => return error_result("Missing required parameter: name"),
        };

        let id = uuid::Uuid::new_v4().to_string();
        let campaign_type = get_str(args, "type").unwrap_or_else(|| "email".into());
        let target_criteria = args
            .get("target_criteria")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        let start_date = get_str(args, "start_date");

        let started_at: Option<chrono::DateTime<chrono::Utc>> = start_date.and_then(|s| {
            s.parse::<chrono::DateTime<chrono::Utc>>().ok()
        });

        match sqlx::query_as::<_, Campaign>(
            "INSERT INTO campaigns.campaigns (id, name, campaign_type, target_criteria, started_at) \
             VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(&id)
        .bind(&name)
        .bind(&campaign_type)
        .bind(&target_criteria)
        .bind(&started_at)
        .fetch_one(self.db.pool())
        .await
        {
            Ok(campaign) => {
                self.log_event(&id, "created", Some(&format!("Campaign '{name}' created"))).await;
                info!(id = id, name = name, "Created campaign");
                json_result(&campaign)
            }
            Err(e) => error_result(&format!("Failed to create campaign: {e}")),
        }
    }

    async fn handle_add_recipients(&self, args: &serde_json::Value) -> CallToolResult {
        let campaign_id = match get_str(args, "campaign_id") {
            Some(i) => i,
            None => return error_result("Missing required parameter: campaign_id"),
        };
        let emails = get_str_array(args, "emails");
        if emails.is_empty() {
            return error_result("Missing required parameter: emails (must be non-empty array)");
        }

        // Verify campaign exists
        let exists: Option<(String,)> =
            match sqlx::query_as("SELECT id FROM campaigns.campaigns WHERE id = $1")
                .bind(&campaign_id)
                .fetch_optional(self.db.pool())
                .await
            {
                Ok(c) => c,
                Err(e) => return error_result(&format!("Database error: {e}")),
            };
        if exists.is_none() {
            return error_result(&format!("Campaign '{campaign_id}' not found"));
        }

        let mut added = 0i64;
        let mut skipped = 0i64;

        for email in &emails {
            let id = uuid::Uuid::new_v4().to_string();
            match sqlx::query(
                "INSERT INTO campaigns.recipients (id, campaign_id, contact_email) \
                 VALUES ($1, $2, $3) ON CONFLICT (campaign_id, contact_email) DO NOTHING",
            )
            .bind(&id)
            .bind(&campaign_id)
            .bind(email)
            .execute(self.db.pool())
            .await
            {
                Ok(r) => {
                    if r.rows_affected() > 0 {
                        added += 1;
                    } else {
                        skipped += 1;
                    }
                }
                Err(e) => return error_result(&format!("Failed to add recipient {email}: {e}")),
            }
        }

        self.log_event(
            &campaign_id,
            "recipients_added",
            Some(&format!("{added} added, {skipped} skipped (duplicates)")),
        )
        .await;

        info!(campaign_id = campaign_id, added = added, "Added recipients");
        json_result(&serde_json::json!({
            "campaign_id": campaign_id,
            "added": added,
            "skipped": skipped,
            "total_requested": emails.len()
        }))
    }

    async fn handle_launch_campaign(&self, campaign_id: &str) -> CallToolResult {
        let campaign: Option<Campaign> =
            match sqlx::query_as("SELECT * FROM campaigns.campaigns WHERE id = $1")
                .bind(campaign_id)
                .fetch_optional(self.db.pool())
                .await
            {
                Ok(c) => c,
                Err(e) => return error_result(&format!("Database error: {e}")),
            };

        let campaign = match campaign {
            Some(c) => c,
            None => return error_result(&format!("Campaign '{campaign_id}' not found")),
        };

        if campaign.status != "draft" && campaign.status != "paused" {
            return error_result(&format!(
                "Cannot launch campaign in '{}' status. Must be 'draft' or 'paused'.",
                campaign.status
            ));
        }

        match sqlx::query_as::<_, Campaign>(
            "UPDATE campaigns.campaigns SET status = 'active', started_at = COALESCE(started_at, now()) \
             WHERE id = $1 RETURNING *",
        )
        .bind(campaign_id)
        .fetch_one(self.db.pool())
        .await
        {
            Ok(updated) => {
                self.log_event(campaign_id, "launched", Some("Campaign activated")).await;
                info!(id = campaign_id, "Launched campaign");
                json_result(&updated)
            }
            Err(e) => error_result(&format!("Failed to launch campaign: {e}")),
        }
    }

    async fn handle_pause_campaign(&self, campaign_id: &str) -> CallToolResult {
        let campaign: Option<Campaign> =
            match sqlx::query_as("SELECT * FROM campaigns.campaigns WHERE id = $1")
                .bind(campaign_id)
                .fetch_optional(self.db.pool())
                .await
            {
                Ok(c) => c,
                Err(e) => return error_result(&format!("Database error: {e}")),
            };

        let campaign = match campaign {
            Some(c) => c,
            None => return error_result(&format!("Campaign '{campaign_id}' not found")),
        };

        if campaign.status != "active" {
            return error_result(&format!(
                "Cannot pause campaign in '{}' status. Must be 'active'.",
                campaign.status
            ));
        }

        match sqlx::query_as::<_, Campaign>(
            "UPDATE campaigns.campaigns SET status = 'paused' WHERE id = $1 RETURNING *",
        )
        .bind(campaign_id)
        .fetch_one(self.db.pool())
        .await
        {
            Ok(updated) => {
                self.log_event(campaign_id, "paused", Some("Campaign paused")).await;
                info!(id = campaign_id, "Paused campaign");
                json_result(&updated)
            }
            Err(e) => error_result(&format!("Failed to pause campaign: {e}")),
        }
    }

    async fn handle_campaign_metrics(&self, campaign_id: &str) -> CallToolResult {
        let campaign: Option<Campaign> =
            match sqlx::query_as("SELECT * FROM campaigns.campaigns WHERE id = $1")
                .bind(campaign_id)
                .fetch_optional(self.db.pool())
                .await
            {
                Ok(c) => c,
                Err(e) => return error_result(&format!("Database error: {e}")),
            };

        let campaign = match campaign {
            Some(c) => c,
            None => return error_result(&format!("Campaign '{campaign_id}' not found")),
        };

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM campaigns.recipients WHERE campaign_id = $1",
        )
        .bind(campaign_id)
        .fetch_one(self.db.pool())
        .await
        .unwrap_or((0,));

        let sent: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM campaigns.recipients WHERE campaign_id = $1 AND sent_at IS NOT NULL",
        )
        .bind(campaign_id)
        .fetch_one(self.db.pool())
        .await
        .unwrap_or((0,));

        let opened: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM campaigns.recipients WHERE campaign_id = $1 AND opened_at IS NOT NULL",
        )
        .bind(campaign_id)
        .fetch_one(self.db.pool())
        .await
        .unwrap_or((0,));

        let clicked: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM campaigns.recipients WHERE campaign_id = $1 AND clicked_at IS NOT NULL",
        )
        .bind(campaign_id)
        .fetch_one(self.db.pool())
        .await
        .unwrap_or((0,));

        let replied: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM campaigns.recipients WHERE campaign_id = $1 AND replied_at IS NOT NULL",
        )
        .bind(campaign_id)
        .fetch_one(self.db.pool())
        .await
        .unwrap_or((0,));

        let bounced: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM campaigns.recipients WHERE campaign_id = $1 AND status = 'bounced'",
        )
        .bind(campaign_id)
        .fetch_one(self.db.pool())
        .await
        .unwrap_or((0,));

        let total_f = if total.0 > 0 { total.0 as f64 } else { 1.0 };

        json_result(&CampaignMetrics {
            campaign_id: campaign_id.to_string(),
            campaign_name: campaign.name,
            total_recipients: total.0,
            sent: sent.0,
            opened: opened.0,
            clicked: clicked.0,
            replied: replied.0,
            bounced: bounced.0,
            open_rate: (opened.0 as f64 / total_f * 100.0 * 10.0).round() / 10.0,
            click_rate: (clicked.0 as f64 / total_f * 100.0 * 10.0).round() / 10.0,
            reply_rate: (replied.0 as f64 / total_f * 100.0 * 10.0).round() / 10.0,
            bounce_rate: (bounced.0 as f64 / total_f * 100.0 * 10.0).round() / 10.0,
        })
    }

    async fn handle_ab_test(&self, args: &serde_json::Value) -> CallToolResult {
        let campaign_id = match get_str(args, "campaign_id") {
            Some(i) => i,
            None => return error_result("Missing required parameter: campaign_id"),
        };
        let name = match get_str(args, "name") {
            Some(n) => n,
            None => return error_result("Missing required parameter: name"),
        };
        let subject = match get_str(args, "subject") {
            Some(s) => s,
            None => return error_result("Missing required parameter: subject"),
        };
        let body = match get_str(args, "body") {
            Some(b) => b,
            None => return error_result("Missing required parameter: body"),
        };
        let recipient_pct = get_f64(args, "recipient_pct").unwrap_or(50.0);

        // Verify campaign exists
        let exists: Option<(String,)> =
            match sqlx::query_as("SELECT id FROM campaigns.campaigns WHERE id = $1")
                .bind(&campaign_id)
                .fetch_optional(self.db.pool())
                .await
            {
                Ok(c) => c,
                Err(e) => return error_result(&format!("Database error: {e}")),
            };
        if exists.is_none() {
            return error_result(&format!("Campaign '{campaign_id}' not found"));
        }

        let id = uuid::Uuid::new_v4().to_string();

        match sqlx::query_as::<_, Variant>(
            "INSERT INTO campaigns.variants (id, campaign_id, name, subject, body, recipient_pct) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        )
        .bind(&id)
        .bind(&campaign_id)
        .bind(&name)
        .bind(&subject)
        .bind(&body)
        .bind(recipient_pct)
        .fetch_one(self.db.pool())
        .await
        {
            Ok(variant) => {
                self.log_event(
                    &campaign_id,
                    "variant_created",
                    Some(&format!("Variant '{name}' at {recipient_pct}%")),
                )
                .await;
                info!(campaign_id = campaign_id, variant = name, "Created A/B variant");
                json_result(&variant)
            }
            Err(e) => error_result(&format!("Failed to create variant: {e}")),
        }
    }

    async fn handle_list_campaigns(&self, args: &serde_json::Value) -> CallToolResult {
        let status = get_str(args, "status");
        let campaign_type = get_str(args, "type");
        let limit = Self::get_i64(args, "limit").unwrap_or(50);

        let mut sql = String::from(
            "SELECT id, name, campaign_type, target_criteria, status, started_at, created_at \
             FROM campaigns.campaigns WHERE 1=1",
        );
        let mut param_idx = 1u32;
        let mut params: Vec<String> = Vec::new();

        if let Some(ref s) = status {
            sql.push_str(&format!(" AND status = ${param_idx}"));
            param_idx += 1;
            params.push(s.clone());
        }
        if let Some(ref t) = campaign_type {
            sql.push_str(&format!(" AND campaign_type = ${param_idx}"));
            param_idx += 1;
            params.push(t.clone());
        }

        sql.push_str(&format!(" ORDER BY created_at DESC LIMIT ${param_idx}"));

        let mut query = sqlx::query_as::<_, Campaign>(&sql);
        for p in &params {
            query = query.bind(p);
        }
        query = query.bind(limit);

        match query.fetch_all(self.db.pool()).await {
            Ok(campaigns) => json_result(&campaigns),
            Err(e) => error_result(&format!("Database error: {e}")),
        }
    }

    async fn handle_campaign_timeline(&self, campaign_id: &str, limit: i64) -> CallToolResult {
        // Verify campaign exists
        let exists: Option<(String,)> =
            match sqlx::query_as("SELECT id FROM campaigns.campaigns WHERE id = $1")
                .bind(campaign_id)
                .fetch_optional(self.db.pool())
                .await
            {
                Ok(c) => c,
                Err(e) => return error_result(&format!("Database error: {e}")),
            };
        if exists.is_none() {
            return error_result(&format!("Campaign '{campaign_id}' not found"));
        }

        match sqlx::query_as::<_, CampaignEvent>(
            "SELECT * FROM campaigns.events WHERE campaign_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(campaign_id)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        {
            Ok(events) => json_result(&events),
            Err(e) => error_result(&format!("Database error: {e}")),
        }
    }
}

// ============================================================================
// ServerHandler trait implementation
// ============================================================================

impl ServerHandler for CampaignMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "DataXLR8 Campaign MCP — create, manage, and track email/LinkedIn/multi-channel campaigns with A/B testing"
                    .into(),
            ),
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, rmcp::ErrorData>> + Send + '_
    {
        async {
            Ok(ListToolsResult {
                tools: build_tools(),
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, rmcp::ErrorData>> + Send + '_
    {
        async move {
            let args =
                serde_json::to_value(&request.arguments).unwrap_or(serde_json::Value::Null);
            let name_str: &str = request.name.as_ref();

            let result = match name_str {
                "create_campaign" => self.handle_create_campaign(&args).await,
                "add_recipients" => self.handle_add_recipients(&args).await,
                "launch_campaign" => match get_str(&args, "campaign_id") {
                    Some(id) => self.handle_launch_campaign(&id).await,
                    None => error_result("Missing required parameter: campaign_id"),
                },
                "pause_campaign" => match get_str(&args, "campaign_id") {
                    Some(id) => self.handle_pause_campaign(&id).await,
                    None => error_result("Missing required parameter: campaign_id"),
                },
                "campaign_metrics" => match get_str(&args, "campaign_id") {
                    Some(id) => self.handle_campaign_metrics(&id).await,
                    None => error_result("Missing required parameter: campaign_id"),
                },
                "ab_test" => self.handle_ab_test(&args).await,
                "list_campaigns" => self.handle_list_campaigns(&args).await,
                "campaign_timeline" => match get_str(&args, "campaign_id") {
                    Some(id) => {
                        let limit = Self::get_i64(&args, "limit").unwrap_or(100);
                        self.handle_campaign_timeline(&id, limit).await
                    }
                    None => error_result("Missing required parameter: campaign_id"),
                },
                _ => error_result(&format!("Unknown tool: {}", request.name)),
            };

            Ok(result)
        }
    }
}
