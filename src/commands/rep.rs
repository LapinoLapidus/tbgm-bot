use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;
use serenity::model::prelude::{application_command::*, *};
use crate::util;

pub async fn rep_command(ctx: &Context, command: &ApplicationCommandInteraction) -> anyhow::Result<(), String> {
    let user = command.data.options.get(0).unwrap().resolved.as_ref().expect("No user object");
    let (user, member) = if let ApplicationCommandInteractionDataOptionValue::User(user, member) = user {
        (user, member)
    } else {
        return Err("Failed to parse user".parse().unwrap())
    };
    let member = member.as_ref().unwrap();
    let results = util::get_rep_data(user.id.0 as i64).await;
    let results = results.unwrap();
    let (name, avatar) = (user.name.clone(), user.avatar_url().as_ref().unwrap_or(&"".parse().unwrap()).clone());
    command.create_interaction_response(&ctx.http, |response| {
        response.interaction_response_data(|data| {
            data.embed(|embed| {
                embed.title(format!("Reputation of {}#{}", member.nick.as_ref().unwrap_or_else(|| &name), user.discriminator ))
                    .footer(|footer| footer.text("Developed by Ziikmar#3262"))
                    .field("+rep", format!("+{}", results.positive), true)
                    .field("-rep", format!("-{}", results.negative), true)
                    .image(avatar)
            })
        })
    }).await.expect("Failed to send rep message");
    Ok(())
}