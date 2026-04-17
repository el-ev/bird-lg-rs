use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use common::api::AppResponse;
use futures::channel::oneshot;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{Event, EventSource, MessageEvent};

fn is_terminal_response(response: &AppResponse) -> bool {
    matches!(
        response,
        AppResponse::TracerouteDone { .. }
            | AppResponse::TracerouteError { .. }
            | AppResponse::PingDone { .. }
            | AppResponse::PingError { .. }
            | AppResponse::RouteLookupDone { .. }
            | AppResponse::RouteLookupError { .. }
            | AppResponse::ProtocolDetailsDone { .. }
            | AppResponse::ProtocolDetailsError { .. }
            | AppResponse::Error(_)
    )
}

pub async fn consume_app_sse<F>(url: String, on_response: F) -> Result<(), String>
where
    F: FnMut(AppResponse) + 'static,
{
    let source = EventSource::new(&url).map_err(|_| format!("Failed to open SSE stream: {url}"))?;
    let source = Rc::new(source);
    let completed = Rc::new(Cell::new(false));
    let on_response = Rc::new(RefCell::new(on_response));
    let (tx, rx) = oneshot::channel::<Result<(), String>>();
    let tx = Rc::new(RefCell::new(Some(tx)));

    let onmessage = {
        let source = source.clone();
        let completed = completed.clone();
        let on_response = on_response.clone();
        let tx = tx.clone();

        Closure::<dyn FnMut(MessageEvent)>::wrap(Box::new(move |event: MessageEvent| {
            let data = event.data().as_string().unwrap_or_default();

            match serde_json::from_str::<AppResponse>(&data) {
                Ok(response) => {
                    let terminal = is_terminal_response(&response);
                    on_response.borrow_mut()(response);

                    if terminal && !completed.replace(true) {
                        source.close();
                        if let Some(sender) = tx.borrow_mut().take() {
                            let _ = sender.send(Ok(()));
                        }
                    }
                }
                Err(error) => {
                    if !completed.replace(true) {
                        source.close();
                        if let Some(sender) = tx.borrow_mut().take() {
                            let _ = sender
                                .send(Err(format!("Failed to decode SSE response: {}", error)));
                        }
                    }
                }
            }
        }))
    };

    let onerror = {
        let source = source.clone();
        let completed = completed.clone();
        let tx = tx.clone();

        Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_event: Event| {
            if completed.get() {
                return;
            }

            completed.set(true);
            source.close();
            if let Some(sender) = tx.borrow_mut().take() {
                let _ = sender.send(Err("SSE connection closed before completion".to_string()));
            }
        }))
    };

    source.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    source.set_onerror(Some(onerror.as_ref().unchecked_ref()));

    let result = rx
        .await
        .unwrap_or_else(|_| Err("SSE completion channel dropped".to_string()));

    source.close();
    source.set_onmessage(None);
    source.set_onerror(None);
    drop(onmessage);
    drop(onerror);

    result
}
