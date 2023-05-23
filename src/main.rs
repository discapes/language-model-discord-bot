use std::{env};
use std::sync::Arc;

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
            println!("Received command interaction: {:#?}", command);

            let ctx = Arc::new(ctx);
            let ctx_clone = ctx.clone();
            let command = Arc::new(command);
            let command_clone = command.clone();

            tokio::spawn(async move {
                command_clone.create_interaction_response(&ctx_clone.http, |response| {
                    response.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                }).await
            });

            let message = cast!(
                command.data.options.get(0).unwrap().resolved.clone().unwrap(),
                CommandDataOptionValue::String
            );

            let content = match command.data.name.as_str() {
                "chat" => match api::send_request(
                    &env::var("API_URL").expect("Expected API_URL in the environment"),
                    &Request {
                        user_input: &message,
                        ..Default::default()
                    },
                )
                .await
                {
                    Ok(text) => Ok(text),
                    Err(e) => {
                        eprintln!("Error! {e:?}");
                        Err(e)
                    }
                },

                _ => Ok("not implemented :(".to_string()),
            };

            if content.is_err() {
                return;
            };
            let mut content = content.unwrap();
            content.insert_str(0, &format!("<@{}>: **{}**\n\n", command.user.id, message));

            if let Err(why) = command
                .edit_original_interaction_response(&ctx.http, |response| response.content(content))
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let global_command =
            Command::create_global_application_command(&ctx.http, register_command).await;

        println!(
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
        println!("Client error: {:?}", why);
    }
}
