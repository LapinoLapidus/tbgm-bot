use std::ops::Add;
use std::panic;
use chrono::{Duration, NaiveDate, TimeZone, Utc};
use num_format::{Locale, ToFormattedString};
use serenity::model::prelude::application_command::*;
use serenity::prelude::*;
use crate::util;
use crate::util::ContributionLogs;

#[derive(Ord, Eq, PartialEq, PartialOrd, Clone)]
struct MemberXP {
    xp: u64,
    uuid: String
}

pub async fn leaderboard_command(ctx: &Context, command: &ApplicationCommandInteraction) -> anyhow::Result<(), String> {
    let guild = util::get_guild("The Broken Gasmask").await.unwrap();
    let mut show_date_warning = false;
    let mut custom_date = false;
    let start_date = if let Some(date) = command.data.options.first() {
        let date = date.value.as_ref().unwrap().as_str().unwrap();
        let date = NaiveDate::parse_from_str(date, "%Y-%m-%d");
        let date = if let Err(_) = date {
            return Err("Wrong date format".parse().unwrap());
        } else {
            Utc.from_local_date(&date.unwrap()).unwrap()
        };
        let cutoff_date = Utc.from_local_date(&NaiveDate::parse_from_str("2022-07-13", "%Y-%m-%d").unwrap()).unwrap();
        if cutoff_date > date {
            show_date_warning = true;
        }
        custom_date = true;
        date
    } else {
        Utc::today() - Duration::days(7)
    };
    let mut results = vec![];
    for member in guild.members {
        let log_7d = util::get_contribution_log(&member.uuid, start_date).await;
        let starting_xp = log_7d.unwrap_or_else(|_| ContributionLogs {..Default::default()}).contributed_xp;
        let result = panic::catch_unwind(|| {
            member.contributed - starting_xp as u64
        });
        if result.is_ok() {
            results.push(MemberXP {
                xp: result.unwrap(),
                uuid: member.name.as_str().parse().unwrap()
            });
        }

    }
    results.sort();
    results.reverse();
    let results = results[0..10].to_vec();
    let mut description = String::from("");
    if show_date_warning {
        description = "WARNING: Date shown is before 2022-07-13, so the total contributed XP will be shown instead.\n".parse().unwrap()
    }
    let mut i = 1;
    for result in results {
        description = description.add(&*format!("{}. **{}**: {}", i, result.uuid, result.xp.to_formatted_string(&Locale::en))).add("\n");
        i+=1;
    }
    let title = match custom_date {
        false => String::from("XP Leaderboard for the last week."),
        true => String::from("XP Leaderboard since: ") + &*String::from(start_date.to_string())
    };
    command.create_interaction_response(&ctx.http, |response| {
        response.interaction_response_data(|data| data.embed(|embed| embed.title(title).description(description)))
    }).await.expect("TODO: panic message");
    Ok(())
}