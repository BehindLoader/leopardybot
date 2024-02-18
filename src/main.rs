use leopardybot::conf::get_config;
use leopardybot::db::{clone_db, create_db};
use leopardybot::error::Result;
use leopardybot::router::base::{CommandScope, RouteCfg, RouteMatch, Router};
use leopardybot::router::help::HelpCommand;
use leopardybot::router::play_group::PlayGroupCommand;
use leopardybot::router::play_single::PlaySingleCommand;
use leopardybot::router::poll_answer::PollAnswerHandler;
use leopardybot::router::restart::RestartCommand;
use leopardybot::router::start::StartCommand;
use leopardybot::telebot::client::Client;
use leopardybot::{job, seeder, server};
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let c = get_config();

    let db = create_db().await?;
    let client = Client::new(
        &c.telegram_token,
        &format!("https://{}/api/v1/telegram", &c.host),
        c.telegram_secret_token.as_ref(),
        c.telegram_max_connections,
        "https://api.telegram.org",
    )
    .await?;
    let router = Router::new()
        .register(RouteCfg {
            route_match: RouteMatch::Command {
                command: "/help".to_owned(),
                scope: CommandScope::Any,
            },
            handler: Box::new(HelpCommand),
            description: Some("Вывести список доступных команд".to_owned()),
        })
        .register(RouteCfg {
            route_match: RouteMatch::Command {
                command: "/start".to_owned(),
                scope: CommandScope::Any,
            },
            handler: Box::new(StartCommand),
            description: None,
        })
        .register(RouteCfg {
            route_match: RouteMatch::Command {
                command: "/play".to_owned(),
                scope: CommandScope::GroupChats,
            },
            handler: Box::new(PlayGroupCommand),
            description: Some("Начать групповую игру".to_owned()),
        })
        .register(RouteCfg {
            route_match: RouteMatch::Command {
                command: "/play".to_owned(),
                scope: CommandScope::PrivateChats,
            },
            handler: Box::new(PlaySingleCommand),
            description: Some("Начать одиночную игру".to_owned()),
        })
        .register(RouteCfg {
            route_match: RouteMatch::Command {
                command: "/restart".to_owned(),
                scope: CommandScope::PrivateChats,
            },
            handler: Box::new(RestartCommand),
            description: Some("Пересоздать последний опрос".to_owned()),
        })
        .register(RouteCfg {
            route_match: RouteMatch::PollAnswer,
            handler: Box::new(PollAnswerHandler),
            description: None,
        });

    let commands = router.list_commands()?;
    if !commands.private_chats.is_empty() {
        client
            .set_my_commands(commands.private_chats, "all_private_chats")
            .await?;
    }
    if !commands.group_chats.is_empty() {
        client
            .set_my_commands(commands.group_chats, "all_group_chats")
            .await?;
    }
    if !commands.group_administrators.is_empty() {
        client
            .set_my_commands(commands.group_administrators, "all_chat_administrators")
            .await?;
    }

    seeder::run(clone_db(&db)?).await?;
    job::run(clone_db(&db)?, client.clone()).await;
    server::run(db, client, router).await?;
    Ok(())
}
