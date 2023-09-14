use std::net::SocketAddr;

use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Form, Router,
};
use lazy_static::lazy_static;
use serde::Deserialize;
use volo::FastStr;
lazy_static! {
    static ref CLIENT: volo_gen::volo::redis::RedisClient = {
        let addr: SocketAddr = "127.0.0.1:19260".parse().unwrap();
        volo_gen::volo::redis::RedisClientBuilder::new("redis-client")
            .address(addr)
            .build()
    };
}

async fn get_cache(key: &str) -> Option<String> {
    let req = volo_gen::volo::redis::GetRequest {
        key: FastStr::new(key),
    };
    let resp = CLIENT.get(req).await;
    match resp {
        Ok(resp) => resp.value.map(|s| s.as_str().to_string()),
        Err(e) => {
            tracing::error!("{:?}", e);
            None
        }
    }
}

async fn set_cache(key: &str, value: &str, ttl: Option<i32>) {
    let req = volo_gen::volo::redis::SetRequest {
        key: FastStr::new(key),
        value: FastStr::new(value),
        ttl,
    };
    let resp = CLIENT.set(req).await;
    match resp {
        Ok(resp) => {
            if !resp.success {
                tracing::error!("set failed");
            }
        }
        Err(e) => tracing::error!("{:?}", e),
    }
}

async fn del_cache(key: &str) {
    let req = volo_gen::volo::redis::DelRequest {
        key: FastStr::new(key),
    };
    let resp = CLIENT.del(req).await;
    match resp {
        Ok(resp) => {
            if !resp.success {
                tracing::error!("del failed");
            }
        }
        Err(e) => tracing::error!("{:?}", e),
    }
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build the application with router
    let app = Router::new()
        .route("/ping", get(ping))
        .route("/get/:keys", get(get_key))
        .route("/set", get(show_set_form).post(set_key))
        .route("/del", get(show_del_form).post(del_key));

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ping() -> (StatusCode, &'static str) {
    (StatusCode::OK, "pong")
}

/// Get a key
async fn get_key(Path(key): Path<String>) -> Response {
    match get_cache(key.as_str()).await {
        Some(value) => (StatusCode::OK, value).into_response(),
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

#[derive(Deserialize, Debug)]
struct FormKey {
    key: String,
    value: String,
    ttl: Option<i32>,
}

/// Show the form for set a key
async fn show_set_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/set" method="post">
                    <label for="key">
                        Enter key:
                        <input type="text" name="key">
                    </label>
                    <label for="value">
                        Enter value:
                        <input type="text" name="value">
                    </label>
                    <label for="ttl">
                        Enter ttl:
                        <input type="text" name="ttl">
                    </label>
                    <input type="submit" value="Subscribe!">
                </form>
            </body>
        </html>
        "#,
    )
}

/// Set a key
async fn set_key(Form(setkey): Form<FormKey>) -> Response {
    set_cache(setkey.key.as_str(), setkey.value.as_str(), setkey.ttl).await;
    (StatusCode::OK, "set ok").into_response()
}

#[derive(Deserialize, Debug)]
struct DelFormKey {
    key: String,
}
async fn show_del_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/del" method="post">
                    <label for="key">
                        Enter key:
                        <input type="text" name="key">
                    </label>
                    <input type="submit" value="Subscribe!">
                </form>
            </body>
        </html>
        "#,
    )
}

async fn del_key(Form(delkey): Form<DelFormKey>) -> (StatusCode, &'static str) {
    del_cache(delkey.key.as_str()).await;
    (StatusCode::OK, "del ok")
}
