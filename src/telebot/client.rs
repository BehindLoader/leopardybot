use crate::error::{Error, Result};
use crate::telebot::typings::output::{BotCommand, Message};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct JsonResponse<R> {
    pub ok: bool,
    pub result: Option<R>,
    pub error_code: Option<usize>,
    pub description: Option<String>,
}

impl<R: Debug> JsonResponse<R> {
    pub(crate) fn into_result(self) -> Result<R> {
        let err = format!("{:?}", self);
        match self.ok {
            true => self
                .result
                .ok_or_else(|| Error(format!("Empty result field. {:?}", err))),
            false => Err(Error(format!("Telegram API error. {:?}", self))),
        }
    }
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

    pub(super) async fn execute<R: Debug + DeserializeOwned>(
        &self,
        method: &str,
        form: &[(&str, String)],
    ) -> Result<R> {
        let response = self
            .client
            .post(format!(
                "{}/bot{}/{}",
                self.telegram_api_base_url, self.token, method
            ))
            .form(&form)
            .send()
            .await?;
        response.json::<JsonResponse<R>>().await?.into_result()
    }

    pub(crate) async fn set_webhook_info(&self, url: &str, max_connections: u8) -> Result<bool> {
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

    pub async fn set_my_commands(&self, commands: Vec<BotCommand>, scope: &str) -> Result<bool> {
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

    pub(crate) async fn send_message(&self, chat_id: isize, text: &str) -> Result<Message> {
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
        explanation: Option<String>,
        correct_option_id: usize,
        open_period: Option<u16>,
    ) -> Result<Message> {
        let mut form = vec![
            ("chat_id", chat_id.to_string()),
            ("question", question.to_string()),
            ("options", serde_json::to_string(options)?),
            ("is_anonymous", "false".to_string()),
            ("type", "quiz".to_string()),
            ("correct_option_id", correct_option_id.to_string()),
            ("protect_content", true.to_string()),
        ];
        if let Some(explanation) = explanation {
            form.push(("explanation", explanation));
        }
        if let Some(open_period) = open_period {
            form.push(("open_period", open_period.to_string()));
        }
        let response = self.execute("sendPoll", &form).await;
        debug!("send_quiz: {:?}", response);
        response
    }

    pub(crate) async fn delete_message(&self, chat_id: isize, message_id: usize) -> Result<bool> {
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
