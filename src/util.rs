use reqwest::Client;
use serde::Deserializer;
use serde_json::Value;
use wynn_api_structs::structs::{User, UserResponse, GuildMember, UuidRequestResponse, Guild};
use chrono::{Date, DateTime, Duration, Utc};
use once_cell::sync::OnceCell;
use sqlx::{Pool, Sqlite};


pub static SQLITE_POOL: OnceCell<Pool<Sqlite>> = OnceCell::new();
pub static DB_CLIENT: DBClient = DBClient {};

#[derive(Clone, Copy)]
pub struct DBClient;

impl DBClient {
    pub fn get() -> &'static DBClient {
        &DB_CLIENT
    }

    pub async fn mysql_pool(self) -> &'static Pool<Sqlite> {
        SQLITE_POOL.get().unwrap()
    }
}

#[derive(Debug)]
pub enum RepType {
    PlusRep,
    MinRep,
}

impl RepType {
    fn value(&self) -> u8 {
        match *self {
            RepType::PlusRep => 1,
            RepType::MinRep => 0
        }
    }
}


pub async fn get_uuid_from_username(username: &str) -> Option<String> {
    if let Ok(result) = reqwest::get(format!("https://api.mojang.com/users/profiles/minecraft/{}", username)).await {
        if !result.status().is_success() || result.content_length().unwrap() == 0 {
            return None
        }

        Some(result.json::<UuidRequestResponse>().await.unwrap().id)
    } else {
        None
    }
}

pub async fn get_guild(guild_name: &str) -> Option<Guild> {
    if let Ok(result) = reqwest::get(format!("https://api.wynncraft.com/public_api.php?action=guildStats&command={}", guild_name)).await {
        // Wynncraft API returns 200 for error (bruh), but a content length of 27 indicates failure.
        if !result.status().is_success() || result.content_length().unwrap() == 27 {
            return None;
        }
        Some(result.json::<Guild>().await.unwrap())

    } else {
        None
    }
}

pub async fn get_user(username: &str) -> Option<User> {
    if let Ok(result) = reqwest::get(format!("https://api.wynncraft.com/v2/player/{}/stats", username)).await {
        if !result.status().is_success() {
            return None;
        }
        Some(result.json::<UserResponse>().await.unwrap().data.first().unwrap().clone())
    } else {
        None
    }
}

pub struct TimePeriod(pub Date<Utc>, pub Date<Utc>);
#[derive(Debug)]
pub struct ContributionLogs {
    pub id: i64,
    pub ts: DateTime<Utc>,
    pub username: String,
    pub uuid: String,
    pub contributed_xp: i64
}

impl Default for ContributionLogs {
    fn default() -> Self {
        ContributionLogs {
            id: 0,
            ts: Utc::now(),
            username: String::from(""),
            uuid: String::from(""),
            contributed_xp: 0
        }
    }
}

pub async fn get_contribution_logs(uuid: &str, time_period: TimePeriod) -> Vec<ContributionLogs> {
    let tp1 = time_period.0.to_string().replace("Z", "").replace("UTC", "");
    let tp2 = time_period.1.to_string().replace("Z", "").replace("UTC", "");

    let entries = sqlx::query_as!(ContributionLogs, "SELECT id, ts AS \"ts: _\", username, uuid, contributed_xp FROM contribution_log WHERE uuid = ?1 AND ts >= strftime('%s', ?2) AND ts <= strftime('%s', ?3)", uuid, tp1, tp2).fetch_all(SQLITE_POOL.get().unwrap()).await.unwrap();
    entries
}

pub async fn get_contribution_log(uuid: &str, date: Date<Utc>) -> Result<ContributionLogs, sqlx::Error> {
    let date = date.and_hms(0, 0, 0).timestamp_millis() / 1000;
    let entry = sqlx::query_as!(ContributionLogs, "SELECT id, ts AS \"ts: _\", username, uuid, contributed_xp FROM contribution_log WHERE uuid = ?1 AND ts = ?2", uuid, date).fetch_one(SQLITE_POOL.get().unwrap()).await;
    entry
}

pub async fn can_give_rep(target: i64, giver: i64) -> bool {
    let date = (Utc::now() - Duration::days(1)).timestamp_millis() / 1000;
    println!("{}", date);
    let results = sqlx::query!("SELECT * FROM main.rep WHERE user_id = ?1 AND giver_id = ?2 AND ts >= ?3", target, giver, date).fetch_all(SQLITE_POOL.get().unwrap()).await.unwrap();
    println!("Can give rep: {:?}", results);
    results.len() == 0
}

pub async fn give_rep(rep_type: RepType, member: i64, reason: &str, giver: i64) -> anyhow::Result<(), String>{
    println!("Giving rep!");
    let rep_type = rep_type.value();
    let result = sqlx::query!("INSERT INTO main.rep (user_id, pn, reason, giver_id) VALUES (?1, ?2, ?3, ?4)", member, rep_type, reason, giver).execute(SQLITE_POOL.get().unwrap()).await;
    if let Err(a) = result {
        println!("{}", a);
        return Err("An error occurred".parse().unwrap())
    }
    println!("Success!");
    println!("{}", result.unwrap().last_insert_rowid());
    return Ok(())
}

/// Positive, Negative
#[derive(Debug)]
pub struct RepData {
    pub positive: i32,
    pub negative: i32
}
impl Default for RepData {
    fn default() -> Self {
        RepData {
            positive: 0,
            negative: 0
        }
    }
}

pub async fn get_rep_data(member: i64) -> anyhow::Result<RepData, String> {
    let entry = sqlx::query_as!(RepData, "SELECT x.positive, y.negative FROM (SELECT COUNT(id) AS \"positive\" FROM rep WHERE user_id = ?1 AND pn = 1) AS x, (SELECT COUNT(id) AS \"negative\" FROM rep WHERE user_id = ?1 AND pn = 0) AS y", member).fetch_one(SQLITE_POOL.get().unwrap()).await;
    let entry = entry.unwrap_or(RepData::default());
    Ok(entry)
}