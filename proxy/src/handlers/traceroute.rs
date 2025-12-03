use std::sync::Arc;

use axum::{
    Extension,
    body::Body,
    extract::Query,
    response::{IntoResponse, Response},
};
use common::utils::validate_target;
use serde::Deserialize;
use tokio::io::AsyncReadExt;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::{error, info, warn};

use crate::{
    config::Config,
    services::traceroute::{IpVersion, build_traceroute_command},
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

                        let text_stream = combined_stream.map(move |line| match line {
                            Ok(mut raw_line) => {
                                if !raw_line.ends_with('\n') {
                                    raw_line.push('\n');
                                }
                                Ok::<_, std::io::Error>(raw_line)
                            }
                            Err(e) => {
                                error!(error = %e, %stream_target, "Failed to read traceroute output");
                                Ok(String::new())
                            }
                        });

                        Body::from_stream(text_stream).into_response()
                    }
                    None => {
                        let mut stderr_output = String::new();
                        if let Some(ref mut stderr_reader) = stderr {
                            let _ = stderr_reader.read_to_string(&mut stderr_output).await;
                        }
                        let _ = child.wait().await;

                        warn!(%target, stderr = %stderr_output.trim(), "Traceroute produced no stdout");

                        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, stderr_output)
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
