use serde::Serialize;
use worker::*;

// TODO move to common
const CONFIG_PATH: &str = "/config.json";

#[derive(Serialize)]
struct ConfigResponse {
    backend_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn json_response<T: Serialize>(body: &T, status: u16) -> Result<Response> {
    let json = serde_json::to_string(body)
        .map_err(|e| Error::RustError(format!("Failed to serialize JSON: {}", e)))?;

    Ok(
        Response::from_json(&serde_json::from_str::<serde_json::Value>(&json)?)?
            .with_status(status),
    )
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;

    if url.path() == CONFIG_PATH {
        let backend_url = env.var("BACKEND_URL");
        let username = env.var("USERNAME").ok().map(|v| v.to_string());

        match backend_url {
            Ok(backend_url) => {
                let config = ConfigResponse {
                    backend_url: backend_url.to_string(),
                    username,
                };
                return json_response(&config, 200);
            }
            Err(_) => {
                let error = ErrorResponse {
                    error: "BACKEND_URL is not configured".to_string(),
                };
                return json_response(&error, 500);
            }
        }
    }

    match env.service("ASSETS") {
        Ok(assets) => {
            let asset_response = assets.fetch_request(req).await;

            match asset_response {
                Ok(response) => {
                    if response.status_code() == 404 {
                        let mut index_url = url.clone();
                        index_url.set_path("/index.html");

                        let index_req =
                            Request::new_with_init(index_url.as_str(), &RequestInit::new())?;

                        match assets.fetch_request(index_req).await {
                            Ok(fallback) if fallback.status_code() == 200 => {
                                return Ok(fallback);
                            }
                            _ => {}
                        }
                    }
                    Ok(response)
                }
                Err(e) => {
                    console_error!("Static asset fetch failed: {:?}", e);
                    Response::error("Not Found", 404)
                }
            }
        }
        Err(e) => {
            console_error!("Failed to get ASSETS service: {:?}", e);
            Response::error("Not Found", 404)
        }
    }
}
