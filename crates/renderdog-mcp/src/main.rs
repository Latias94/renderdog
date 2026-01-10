mod paths;
mod server;
mod types;

use std::io::IsTerminal;

use rmcp::{ServiceExt, transport::stdio};

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(false).init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    if std::io::stdin().is_terminal() {
        eprintln!(
            "renderdog-mcp is an MCP stdio server.\n\
It is meant to be launched by an MCP client (Claude Code / Codex / Gemini CLI).\n\
See the workspace README for setup: https://github.com/Latias94/renderdog\n"
        );
    }

    let server = server::RenderdogMcpServer::new();
    let service = match server.serve(stdio()).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "renderdog-mcp failed to start. If you ran it directly, make sure an MCP client is launching it.\n\
Error: {e}"
            );
            return Err(e.into());
        }
    };

    if let Err(e) = service.waiting().await {
        eprintln!(
            "renderdog-mcp stopped. If you ran it directly, this usually means stdin was closed.\n\
Launch it via an MCP client (stdio transport).\n\
Error: {e}"
        );
        return Err(e.into());
    }
    Ok(())
}
