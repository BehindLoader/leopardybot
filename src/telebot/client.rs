use crate::error::Result;
use crate::telebot::typings::output::{BotCommand, Message};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct JsonResponse<R> {
    pub ok: bool,
    pub result: Option<R>,
    pub error_code: Option<usize>,
    pub description: Option<String>,
}

#[derive(Clone)]
pub struct Client {
    token: String,
    client: reqwest::Client,
    secret_token: String,
    telegram_api_base_url: String,
}

impl Client {
    pub async fn new(
        token: &str,
        url: &str,
        secret_token: Option<&String>,
        max_connection: u8,
        telegram_api_base_url: &str,
    ) -> Result<Self> {
        let c = Self {
            token: token.to_owned(),
            client: reqwest::Client::new(),
            secret_token: secret_token
                .map(|token| token.to_owned())
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            telegram_api_base_url: telegram_api_base_url.to_owned(),
        };

        info!("Updating telegram webhook url to {}", &url);
        c.set_webhook_info(url, max_connection).await?;

        Ok(c)
    }

    pub(crate) fn verify_secret_token(&self, token: &str) -> bool {
        self.secret_token == token
    }

    pub(super) async fn execute<R: DeserializeOwned>(
        &self,
        method: &str,
        form: &[(&str, String)],
    ) -> Result<JsonResponse<R>> {
        let response = self
            .client
            .post(format!(
                "{}/bot{}/{}",
                self.telegram_api_base_url, self.token, method
            ))
            .form(&form)
            .send()
            .await?;
        Ok(response.json().await?)
    }

    // pub(crate) async fn get_webhook_info(
    //     &self,
    // ) -> Result<JsonResponse<telebot::typings::output::WebhookInfo>> {
    //     let response = self
    //         .execute::<telebot::typings::output::WebhookInfo>("getWebhookInfo", &[])
    //         .await;
    //     debug!("get_webhook_info: {:?}", response);
    //     response
    // }

    pub(crate) async fn set_webhook_info(
        &self,
        url: &str,
        max_connections: u8,
    ) -> Result<JsonResponse<bool>> {
        let response = self
            .execute(
                "setWebhook",
                &[
                    ("url", url.to_owned()),
                    ("secret_token", self.secret_token.to_owned()),
                    ("max_connections", max_connections.to_string()),
                ],
            )
            .await;
        debug!("set_webhook_info: {:?}", response);
        response
    }

    pub async fn set_my_commands(
        &self,
        commands: Vec<BotCommand>,
        scope: &str,
    ) -> Result<JsonResponse<bool>> {
        let response = self
            .execute(
                "setMyCommands",
                &[
                    ("commands", serde_json::to_string(&commands)?),
                    ("scope", format!(r#"{{"type": "{}"}}"#, scope)),
                ],
            )
            .await;
        debug!("set_webhook_info: {:?}", response);
        response
    }

    pub(crate) async fn send_message(
        &self,
        chat_id: isize,
        text: &str,
    ) -> Result<JsonResponse<Message>> {
        let response = self
            .execute(
                "sendMessage",
                &[
                    ("chat_id", chat_id.to_string()),
                    ("text", text.to_string()),
                    ("parse_mode", "html".to_string()),
                ],
            )
            .await;
        debug!("send_message: {:?}", response);
        response
    }

    pub(crate) async fn send_quiz(
        &self,
        chat_id: isize,
        question: &String,
        options: &Vec<String>,
        correct_option_id: usize,
        open_period: Option<u16>,
    ) -> Result<JsonResponse<Message>> {
        let mut form = vec![
            ("chat_id", chat_id.to_string()),
            ("question", question.to_string()),
            ("options", serde_json::to_string(options)?),
            ("is_anonymous", "false".to_string()),
            ("type", "quiz".to_string()),
            ("correct_option_id", correct_option_id.to_string()),
            ("protect_content", true.to_string()),
        ];
        if let Some(open_period) = open_period {
            form.push(("open_period", open_period.to_string()));
        }
        let response = self.execute("sendPoll", &form).await;
        debug!("send_quiz: {:?}", response);
        response
    }

    pub(crate) async fn delete_message(
        &self,
        chat_id: isize,
        message_id: usize,
    ) -> Result<JsonResponse<bool>> {
        let response = self
            .execute(
                "deleteMessage",
                &[
                    ("chat_id", chat_id.to_string()),
                    ("message_id", message_id.to_string()),
                ],
            )
            .await;
        debug!("delete_message: {:?}", response);
        response
    }
}
