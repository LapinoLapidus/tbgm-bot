use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;
use serenity::model::prelude::{application_command::*, *};
use crate::util;
use crate::util::RepType;

pub async fn nrep_command(ctx: &Context, command: &ApplicationCommandInteraction) -> anyhow::Result<(), String> {
    let user = command.data.options.get(0).unwrap().resolved.as_ref().expect("No user object");
    let str = command.data.options.get(1).unwrap().resolved.as_ref().expect("No reason object");
    let str = if let ApplicationCommandInteractionDataOptionValue::String(str) = str {
        str
    } else {
        return Err("Failed to parse reason".parse().unwrap())
    };
    let user = if let ApplicationCommandInteractionDataOptionValue::User(user, _) = user {
        user
    } else {
        return Err("Failed to parse user".parse().unwrap())
    };
    if !util::can_give_rep(user.id.0 as i64, command.user.id.0 as i64).await {
        return Err("You have already given this person reputation in the last 24h.".parse().unwrap());
    }
    util::give_rep(RepType::MinRep, user.id.0 as i64, str, command.user.id.0 as i64).await.expect("F");
    Ok(())
}