use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use tokio::{io::AsyncReadExt, process::Child};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};
use tracing::{error, warn};

pub async fn stream_command_output(
    mut child: Child,
    command_name: &'static str,
    target: String,
) -> Response {
    let mut stderr = child.stderr.take();
    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            error!(%target, "{} stdout not captured", command_name);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to capture stdout",
            )
                .into_response();
        }
    };

    let mut lines = FramedRead::new(stdout, LinesCodec::new());

    match lines.next().await {
        Some(first_result) => {
            let combined_stream = tokio_stream::iter(vec![first_result]).chain(lines);
            let stream_target = target.clone();

            let text_stream = combined_stream.map(move |line| match line {
                Ok(mut raw_line) => {
                    if !raw_line.ends_with('\n') {
                        raw_line.push('\n');
                    }
                    Ok::<_, std::io::Error>(raw_line)
                }
                Err(e) => {
                    error!(error = %e, %stream_target, "Failed to read {} output", command_name);
                    Ok(String::new())
                }
            });

            // Spawn a task to wait for child process to finish (reap zombie)
            tokio::spawn(async move {
                let _ = child.wait().await;
            });

            Body::from_stream(text_stream).into_response()
        }
        None => {
            let mut stderr_output = String::new();
            if let Some(ref mut stderr_reader) = stderr {
                let _ = stderr_reader.read_to_string(&mut stderr_output).await;
            }
            let _ = child.wait().await;

            warn!(%target, stderr = %stderr_output.trim(), "{} produced no stdout", command_name);

            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, stderr_output).into_response()
        }
    }
}
