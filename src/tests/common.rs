use crate::telebot::client::Client;
use httpmock::{Method, MockServer};
use serde_json::json;

pub(crate) fn init_mock() -> MockServer {
    let mock = MockServer::start();

    mock.mock(|when, then| {
        when.method(Method::POST).path("/botTOKEN/setWebhook");
        then.status(200).json_body(json!({
            "ok": true,
        }));
    });

    mock
}

pub(crate) async fn init_telebot(mock: MockServer) -> Client {
    Client::new(
        "TOKEN",
        "host",
        Some(&"telegram_secret".to_string()),
        0,
        &mock.base_url(),
    )
    .await
    .unwrap()
}
