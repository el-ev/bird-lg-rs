use std::sync::Arc;

use axum::{
    Extension,
    body::Body,
    extract::Query,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tokio::io::AsyncReadExt;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::{error, info, warn};

use crate::config::Config;
use crate::services::traceroute::{
    IpVersion, TracerouteEntry, build_traceroute_command, validate_target,
};

#[derive(Deserialize)]
pub struct TracerouteQuery {
    target: String,
}

pub async fn traceroute(
    Extension(config): Extension<Arc<Config>>,
    Query(params): Query<TracerouteQuery>,
) -> Response {
    run_traceroute(config, params, IpVersion::Any).await
}

pub async fn traceroute4(
    Extension(config): Extension<Arc<Config>>,
    Query(params): Query<TracerouteQuery>,
) -> Response {
    run_traceroute(config, params, IpVersion::V4).await
}

pub async fn traceroute6(
    Extension(config): Extension<Arc<Config>>,
    Query(params): Query<TracerouteQuery>,
) -> Response {
    run_traceroute(config, params, IpVersion::V6).await
}

// FIXME: Doesn't work on macOS
async fn run_traceroute(
    config: Arc<Config>,
    params: TracerouteQuery,
    version: IpVersion,
) -> Response {
    let target = params.target.trim().to_string();
    if let Err(e) = validate_target(&target) {
        warn!(%target, "Invalid traceroute target: {}", e);
        return (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Invalid target: {}", e),
        )
            .into_response();
    }

    let mut cmd = match build_traceroute_command(&config, &target, version) {
        Some(cmd) => cmd,
        None => {
            error!("Traceroute requested but traceroute_bin not configured");
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "traceroute not configured",
            )
                .into_response();
        }
    };

    info!(%target, version = ?version, "Executing traceroute");
    match cmd.spawn() {
        Ok(mut child) => {
            let mut stderr = child.stderr.take();

            if let Some(stdout) = child.stdout.take() {
                let mut lines = FramedRead::new(stdout, LinesCodec::new());

                match lines.next().await {
                    Some(first_line) => {
                        let combined_stream = tokio_stream::iter(vec![first_line]).chain(lines);
                        let stream_target = target.clone();

                        let json_stream = combined_stream.map(move |line| {
                            line.map_err(|e| {
                                error!(error = %e, %stream_target, "Failed to read traceroute output");
                            })
                            .and_then(|l| {
                                TracerouteEntry::try_from(l.as_str()).map_err(|e| {
                                    warn!(%stream_target, line = %l, "Failed to parse traceroute line: {}", e);
                                })
                            })
                            .and_then(|entry| {
                                serde_json::to_string(&entry)
                                    .map(|json| json + "\n")
                                    .map_err(|e| {
                                        error!(error = %e, %stream_target, "Failed to serialize traceroute entry");
                                    })
                            })
                            .or::<std::io::Error>(Ok(String::new()))
                        });

                        Body::from_stream(json_stream).into_response()
                    }
                    None => {
                        let mut stderr_output = String::new();
                        if let Some(stderr_reader) = stderr.as_mut() {
                            match stderr_reader.read_to_string(&mut stderr_output).await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!(error = %e, %target, "Failed to read traceroute stderr");
                                }
                            }
                        }

                        let response_msg = stderr_output.trim().to_string();

                        if let Err(e) = child.wait().await {
                            error!(error = %e, %target, "Failed to wait for traceroute process");
                        }

                        warn!(%target, stderr = %response_msg, "Traceroute produced stderr without stdout");

                        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, response_msg)
                            .into_response()
                    }
                }
            } else {
                error!(%target, "Traceroute stdout not captured");
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to capture stdout",
                )
                    .into_response()
            }
        }
        Err(e) => {
            error!(error = %e, %target, "Failed to execute traceroute command");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to execute traceroute: {}", e),
            )
                .into_response()
        }
    }
}
