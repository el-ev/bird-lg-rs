use std::sync::Arc;

use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tokio::process::Command;
use tracing::{error, info};

use crate::config::Config;

pub async fn get_wireguard(Extension(config): Extension<Arc<Config>>) -> Response {
    info!("Getting WireGuard status");

    let (program, args) = if let Some(cmd) = &config.wireguard_command {
        let mut parts = cmd.split_whitespace();
        let program = parts.next().unwrap_or("wg");
        let args: Vec<&str> = parts.collect();
        (program.to_string(), args)
    } else {
        ("wg".to_string(), vec!["show", "dump"])
    };

    let output = match Command::new(&program).args(&args).output().await {
        Ok(output) => output,
        Err(e) => {
            error!(error = %e, "Failed to execute wireguard command: {} {:?}", program, args);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to execute wireguard command: {}", e),
            )
                .into_response();
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("WireGuard command failed: {}", stderr);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("WireGuard command failed: {}", stderr),
        )
            .into_response();
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .skip(1)
        .step_by(2)
        .filter_map(|line| {
            let mut fields: Vec<&str> = line.split('\t').collect();
            if fields.len() == 9 {
                fields[2] = "(redacted)";
                Some(fields.join("\t"))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        .into_response()
}
