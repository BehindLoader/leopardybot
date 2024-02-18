use crate::api::handler;
use crate::tests::common::{init_mock, init_telebot};
use actix_web::{test, web, App};
use serde_json::json;

// #[actix_web::test]
// async fn test_help_command() {
//     let mock = init_mock();
//     let telebot = init_telebot(mock).await;
//     let app = test::init_service(
//         App::new()
//             .service(web::resource("/").to(handler))
//             .app_data(web::Data::new(telebot)),
//     )
//     .await;
//     let request = test::TestRequest::default()
//         .set_json(json!({
//             "update_id": 0,
//         }))
//         .to_request();
//     let response = test::call_service(&app, request).await;
//     assert!(response.status().is_success(), "{:?}", response.response());
// }
