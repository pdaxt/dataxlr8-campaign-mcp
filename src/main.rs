use anyhow::Result;
use rmcp::transport::io::stdio;
use rmcp::ServiceExt;
use tracing::info;

mod db;
mod tools;

use tools::CampaignMcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    let config = dataxlr8_mcp_core::Config::from_env("dataxlr8-campaign-mcp")
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    dataxlr8_mcp_core::logging::init(&config.log_level);

    info!(
        server = config.server_name,
        "Starting DataXLR8 Campaign MCP server"
    );

    let database = dataxlr8_mcp_core::Database::connect(&config.database_url)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    db::setup_schema(database.pool()).await?;

    let server = CampaignMcpServer::new(database.clone());

    let transport = stdio();
    let service = server.serve(transport).await?;

    info!("Campaign MCP server connected via stdio");
    service.waiting().await?;

    database.close().await;
    info!("Campaign MCP server shut down");

    Ok(())
}
