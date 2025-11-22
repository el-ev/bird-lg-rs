use std::sync::Arc;

use axum::{
    Extension,
    body::Body,
    extract::Query,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};

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
    let target = params.target.trim();
    if let Err(e) = validate_target(target) {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Invalid target: {}", e),
        )
            .into_response();
    }

    let mut cmd = match build_traceroute_command(&config, target, version) {
        Some(cmd) => cmd,
        None => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "traceroute not configured",
            )
                .into_response();
        }
    };

    // FIXME: stderr not handled
    // TODO: report bad host
    match cmd.spawn() {
        Ok(mut child) => {
            if let Some(stdout) = child.stdout.take() {
                let lines = FramedRead::new(stdout, LinesCodec::new());

                let json_stream = lines.map(|line| match line {
                    Ok(l) => {
                        if let Ok(entry) = TracerouteEntry::try_from(l.as_str()) {
                            match serde_json::to_string(&entry) {
                                Ok(json) => Ok::<_, std::io::Error>(json + "\n"),
                                Err(_) => Ok(String::new()),
                            }
                        } else {
                            Ok(String::new())
                        }
                    }
                    Err(_) => Ok(String::new()),
                });

                Body::from_stream(json_stream).into_response()
            } else {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to capture stdout",
                )
                    .into_response()
            }
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to execute traceroute: {}", e),
        )
            .into_response(),
    }
}
