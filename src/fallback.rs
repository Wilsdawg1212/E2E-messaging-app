use warp::Filter;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct ServerState {
    pub messages: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

pub async fn run_server() {
    let state = ServerState {
        messages: Arc::new(Mutex::new(HashMap::new())),
    };

    let state_filter = warp::any().map(move || state.clone());

    let send_message = warp::post()
        .and(warp::path("send"))
        .and(warp::body::json())
        .and(state_filter.clone())
        .map(|msg: String, state: ServerState| {
            let id = Uuid::new_v4().to_string();
            state.messages.lock().unwrap().insert(id.clone(), msg.into_bytes());
            warp::reply::json(&id)
        });

    let retrieve_message = warp::get()
        .and(warp::path("receive"))
        .and(warp::path::param())
        .and(state_filter)
        .map(|id: String, state: ServerState| {
            let mut messages = state.messages.lock().unwrap();
            if let Some(msg) = messages.remove(&id) {
                warp::reply::json(&String::from_utf8(msg).unwrap())
            } else {
                warp::reply::json(&"Message not found")
            }
        });

    let routes = send_message.or(retrieve_message);

    println!("Fallback server running on 127.0.0.1:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
