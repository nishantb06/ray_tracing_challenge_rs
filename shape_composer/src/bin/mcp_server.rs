use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let run_dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("usage: mcp_server <run-dir>"))?;
    shape_composer::serve_mcp(run_dir).await
}
