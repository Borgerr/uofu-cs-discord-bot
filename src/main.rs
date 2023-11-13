mod args;
mod commands;
mod config;
mod data;
mod lang;
mod memory_regex;
mod reaction_management;
mod text_detection;

use anyhow::Context;
use config::Config;
use data::Data;
use data::PoiseContext;
use dotenvy::dotenv;

use clap::Parser;
use poise::builtins::register_application_commands_buttons;
use poise::serenity_prelude as serenity;
use poise::Event;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().context("Failed to load .env file")?;

    let args = args::Args::parse();
    let config = Config::create_from_file(&args.config).expect("Failed to load config");

    config.save();
    let token = std::env::var("DISCORD_TOKEN").context("Expected a discord token")?;
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                register(),
                commands::change_text_detect_cooldown(),
                commands::create_class_category(),
                commands::add_bot_role(),
                commands::remove_bot_role(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .token(token)
        .intents(
            serenity::GatewayIntents::non_privileged()
                | serenity::GatewayIntents::MESSAGE_CONTENT
                | serenity::GatewayIntents::GUILD_MEMBERS
                | serenity::GatewayIntents::GUILD_MESSAGE_REACTIONS
                | serenity::GatewayIntents::GUILD_MESSAGES,
        )
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    serenity::GuildId::from(1065373537591894086),
                )
                .await?;
                Ok(Data::new(config))
            })
        });

    framework.run().await.context("Failed to start bot")
}

#[poise::command(prefix_command)]
pub async fn register(ctx: PoiseContext<'_>) -> anyhow::Result<()> {
    register_application_commands_buttons(ctx).await?;
    Ok(())
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &Event<'_>,
    framework: poise::FrameworkContext<'_, Data, anyhow::Error>,
    _data: &Data,
) -> anyhow::Result<()> {
    match event {
        Event::Message { new_message } => {
            text_detection::text_detection(ctx, framework.user_data, new_message).await?
        }
        Event::ReactionAdd { add_reaction } => {
            reaction_management::reaction_management(ctx, framework.user_data, add_reaction).await?
        }
        Event::ReactionRemove { removed_reaction } => {
            reaction_management::reaction_management(ctx, framework.user_data, removed_reaction)
                .await?
        }
        _ => {}
    };

    Ok(())
}
