use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use common::utils::validate_target;
use tokio::process::Command;
use tracing::{error, info, warn};

use crate::{config::Config, services::traceroute::IpVersion};

pub async fn run_command<F>(
    config: Arc<Config>,
    target: String,
    version: IpVersion,
    command_name: &'static str,
    not_configured_message: &'static str,
    build: F,
) -> Response
where
    F: Fn(&Config, &str, IpVersion) -> Option<Command>,
{
    let target = target.trim().to_string();
    if let Err(error) = validate_target(&target) {
        warn!(%target, "Invalid {} target: {}", command_name, error);
        return (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Invalid target: {}", error),
        )
            .into_response();
    }

    let mut cmd = match build(&config, &target, version) {
        Some(cmd) => cmd,
        None => {
            error!("{} requested but command is not configured", command_name);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                not_configured_message,
            )
                .into_response();
        }
    };

    info!(%target, version = ?version, "Executing {}", command_name);
    match cmd.spawn() {
        Ok(child) => crate::utils::stream_command_output(child, command_name, target).await,
        Err(error) => {
            error!(error = %error, %target, "Failed to execute {} command", command_name);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to execute {}: {}", command_name, error),
            )
                .into_response()
        }
    }
}

pub fn split_command(
    configured: Option<&str>,
    default_program: &str,
    default_args: &[&str],
) -> (String, Vec<String>) {
    if let Some(configured) = configured {
        let mut parts = configured.split_whitespace();
        let program = parts.next().unwrap_or(default_program).to_string();
        let args = parts.map(|value| value.to_string()).collect();
        (program, args)
    } else {
        (
            default_program.to_string(),
            default_args.iter().map(|value| value.to_string()).collect(),
        )
    }
}
