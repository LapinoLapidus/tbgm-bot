extern crate dotenv;

use dotenv::dotenv;
use std::env;

use serenity::async_trait;
use serenity::model::prelude::application_command::ApplicationCommandOptionType;
use serenity::model::prelude::*;
use serenity::{prelude::*, Client};
use sqlx::{ConnectOptions, SqlitePool};
use crate::util::SQLITE_POOL;

mod commands;
mod util;

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected", ready.user.name);
        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in .env")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );
        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("ping").description("A ping command")
                })
                .create_application_command(|command| {
                    command
                        .name("profile")
                        .description("Shows the profile & stats of the group member")
                        .create_option(|option| {
                            option
                                .name("name")
                                .description("The name of the person to show the profile of")
                                .required(true)
                                .kind(ApplicationCommandOptionType::String)
                        })
                })
                .create_application_command(|command| {
                    command.name("leaderboard")
                        .description("Shows the leaderboard of last week's top XP earners")
                        .create_option(|option| {
                            option.name("start_date").description("The date on which to start the leaderboard. Formatted as YYYY-mm-dd").required(false).kind(ApplicationCommandOptionType::String)
                        })
                })
                .create_application_command(|command| {
                    command.name("prep")
                        .description("Gives +rep to a member.")
                        .create_option(|option| {
                            option.name("member")
                                .description("The member to give +rep to")
                                .kind(ApplicationCommandOptionType::User)
                                .required(true)
                        })
                        .create_option(|option| {
                            option.name("reason")
                                .description("The reason for which you give +rep")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    command.name("nrep")
                        .description("Gives -rep to a member.")
                        .create_option(|option| {
                            option.name("member")
                                .description("The member to give -rep to")
                                .kind(ApplicationCommandOptionType::User)
                                .required(true)
                        }).create_option(|option| {
                            option.name("reason")
                                .description("The reason for which you give +rep")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                    })
                }).create_application_command(|command| {
                command.name("rep")
                    .description("Shows the rep of a member")
                    .create_option(|option| {
                        option.name("member")
                            .description("The member to show the rep of")
                            .kind(ApplicationCommandOptionType::User)
                            .required(true)
                    })
            })
        })
        .await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let result = match command.data.name.as_str() {
                // "ping" => "Hey I'm alive".to_string().,
                "profile" => commands::profile_command(&ctx, &command).await,
                "leaderboard" => commands::leaderboard_command(&ctx, &command).await,
                "prep" => commands::prep_command(&ctx, &command).await,
                "nrep" => commands::nrep_command(&ctx, &command).await,
                "rep" => commands::rep_command(&ctx, &command).await,
                _ => panic!(),
            };

            if let Err(why) = result {
                command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.content(why))
                    })
                    .await.expect("TODO: panic message");
            };
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    SQLITE_POOL.set({
        SqlitePool::connect(env::var("DATABASE_URL").unwrap().as_str()).await.expect("Failed to load Sqlite3 database")
    }).expect("Failed to set connection.");


    let token = env::var("TOKEN")?;
    let mut client = Client::builder(token, GatewayIntents::all())
        .event_handler(Handler)
        .await?;
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
    Ok(())
}
