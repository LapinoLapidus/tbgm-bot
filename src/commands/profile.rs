use chrono::Duration;
use serenity::model::prelude::*;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;
use crate::util;
use num_format::{Locale, ToFormattedString};
use crate::util::TimePeriod;

pub async fn profile_command(ctx: &Context, command: &ApplicationCommandInteraction) -> anyhow::Result<(), String> {
    let profilee = command.data.options.first().unwrap().value.as_ref().unwrap().to_string().replace("\"", "");
    let guild = util::get_guild("The Broken Gasmask").await.unwrap();
    let member = match guild.members.iter().find(|member|member.name.to_lowercase() == profilee.to_lowercase()) {
        Some(member) => member,
        None => return Err(format!("This user is not in {}", "The Broken Gasmask".to_string()))
    };
    let user = match util::get_user(&member.name).await {
        Some(user) => user,
        None => return Err("User does not exist.".to_string())
    };

    let log_24h = util::get_contribution_log(&member.uuid,chrono::offset::Utc::today() - Duration::days(1)).await;
    let log_7d = util::get_contribution_log(&member.uuid, chrono::offset::Utc::today() - Duration::days(7)).await;

    let log_24h = if let Ok(log_24h) = log_24h {
        Some((member.contributed as i64 - log_24h.contributed_xp).to_formatted_string(&Locale::en))
    } else {
        None
    };

    let log_7d = if let Ok(log_7d) = log_7d {
        Some((member.contributed as i64 - log_7d.contributed_xp).to_formatted_string(&Locale::en))
    } else {
        // Use get_contribution_logs.first() if missing
        None
    };


    let mut completed_corrupted = 0;
    user.classes.iter().for_each(|class| class.dungeons.list.iter().for_each(|dungeon| if dungeon.name.starts_with("Corrupted") {completed_corrupted += dungeon.completed}));

    command.create_interaction_response(&ctx.http, |interaction| {
        interaction.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|message| message.embed(|embed| {
            embed.title(format!("Profile of Guild Member: {}", profilee))
                .image(format!("https://visage.surgeplay.com/bust/{}", &member.uuid))
                .field("Rank", some_kind_of_uppercase_first_letter(&member.rank.to_lowercase()), true)
                .field("Join Date", &member.joined_friendly, true)
                .field("Corrupted Dungeon Completes", completed_corrupted.to_string(), true)
                .field("Contributed XP", &member.contributed.to_formatted_string(&Locale::en), true)
                .field("XP Last 24h", log_24h.unwrap_or(String::from("N/A")), true)
                .field("XP Last 7d", log_7d.unwrap_or(String::from("N/A")), true)
        }))
    }).await.expect("Could not send message.");
    Ok(())
}

fn some_kind_of_uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}