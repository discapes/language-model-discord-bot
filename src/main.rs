use std::cmp;
use std::env;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Error;
use env_logger::Env;
use log::{debug, error, info};
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOptionValue;
use serenity::prelude::*;
use serenity::{async_trait, builder};

use crate::api::Request;

mod api;

struct Handler;

macro_rules! cast {
    ($target: expr, $pat: path) => {{
        if let $pat(a) = $target {
            // #1
            a
        } else {
            panic!("mismatch variant when cast to {}", stringify!($pat)); // #2
        }
    }};
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            debug!("Received command interaction: {:#?}", command);

            let ctx = Arc::new(ctx);
            let ctx_clone = ctx.clone();
            let command = Arc::new(command);
            let command_clone = command.clone();

            tokio::spawn(async move {
                command_clone
                    .create_interaction_response(&ctx_clone.http, |response| {
                        response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                    })
                    .await
            });

            let user_input = cast!(
                command
                    .data
                    .options
                    .get(0)
                    .unwrap()
                    .resolved
                    .clone()
                    .unwrap(),
                CommandDataOptionValue::String
            );

            let llm_response = match command.data.name.as_str() {
                "chat" => {
                    api::send_request(
                        &env::var("API_URL").expect("Expected API_URL in the environment"),
                        &Request {
                            user_input: &user_input,
                            ..Default::default()
                        },
                    )
                    .await
                }
                _ => Err(anyhow!("not implemented :(")),
            };

            let errors: Vec<Error> = match llm_response {
                Ok(llm_response) => {
                    info!(
                        "{}#{}: **{}**\n\n{}",
                        command.user.name, command.user.discriminator, user_input, llm_response
                    );
                    let mut msg_content: &str = &format!(
                        "<@{}>: **{}**\n\n{}",
                        command.user.id, user_input, llm_response
                    );
                    let mut errors: Vec<Error> = vec![];
                    let sub_len = 2000;

                    let (chunk, rest) = msg_content.split_at(cmp::min(sub_len, msg_content.len()));
                    msg_content = rest;

                    if let Err(e) = command
                        .create_followup_message(&ctx.http, |response| response.content(chunk))
                        .await
                    {
                        errors.push(Error::from(e));
                    }

                    while !msg_content.is_empty() {
                        let (chunk, rest) =
                            msg_content.split_at(cmp::min(sub_len, msg_content.len()));
                        msg_content = rest;

                        if let Err(e) = command
                            .create_followup_message(&ctx.http, |response| response.content(chunk))
                            .await
                        {
                            errors.push(Error::from(e));
                        }
                    }
                    errors
                }
                Err(e) => {
                    let err_string = e.to_string();
                    let mut errors: Vec<Error> = vec![e];

                    if let Err(second_error) = command
                        .edit_original_interaction_response(&ctx.http, |response| {
                            response.content(format!(
                                "<@{}> maus error: `{err_string}`",
                                std::env::var("MAINTAINER_UID").unwrap_or("?".to_string())
                            ))
                        })
                        .await
                    {
                        errors.push(second_error.into())
                    }
                    errors
                }
            };

            for e in errors {
                error!("{e}")
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let global_command =
            Command::create_global_application_command(&ctx.http, register_command).await;

        debug!(
            "I created the following global slash command: {:#?}",
            global_command
        );
    }
}

pub fn register_command(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("chat")
        .description("Message to send")
        .create_option(|option| {
            option
                .name("message")
                .description("Chat with wizard")
                .kind(CommandOptionType::String)
                .required(true)
        })
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("wizard_bot=info")).init();
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in the environment");

    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Start a single shard, and start listening to events.
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
