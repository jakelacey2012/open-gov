use std::collections::HashMap;
use std::env;
use std::thread;
use std::time::Duration;

use clokwerk::{AsyncScheduler, TimeUnits};
use diesel::prelude::*;
use diesel::PgConnection;
use dotenvy::dotenv;
use serde::ser::StdError;
use serenity::async_trait;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::GuildChannel;
use serenity::model::prelude::GuildId;
use serenity::prelude::*;
use std::error::Error;
use std::sync::Arc;

use serde::Deserialize;
use serenity::prelude::GatewayIntents;
use serenity::Client;

pub mod model;
pub mod schema;

use crate::model::*;
use crate::schema::*;

// type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct OpenGovBotHandler;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // Create new instance of the discord client to handle events
    let mut client = Client::builder(&token, intents)
        .event_handler(OpenGovBotHandler)
        .await
        .expect("Err creating client");

    tokio::spawn(async move {
        let mut scheduler = AsyncScheduler::new();
        scheduler.every(1.minutes()).run(ingest);

        loop {
            scheduler.run_pending().await;
            thread::sleep(Duration::from_millis(10));
        }
    });

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

async fn ingest() {
    let connection = &mut establish_connection();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let http = Arc::new(Http::new(&token));

    for division_obj in fetch_divisions().await.unwrap().into_iter() {
        let load_division = self::schema::divisions::dsl::divisions
            .filter(divisions::division_id.eq(division_obj.division_id))
            .first::<Division>(connection);

        println!(
            "processing division: {:?} - {:?}",
            division_obj.division_id, division_obj.title
        );

        match load_division {
            Err(diesel::result::Error::NotFound) => {
                println!("   division does not currenlty exist.");
                fixture_thread(connection, &http, &division_obj).await
            }
            Ok(division) => {
                println!("   division should be updated.");
                update_thread(connection, &http, &division, &division_obj).await
            }
            Err(_) => todo!("what"),
        }
        .unwrap();
    }
}

async fn update_thread(
    connection: &mut PgConnection,
    http: &Arc<Http>,
    division: &Division,
    division_obj: &DivisionObj,
) -> Result<model::Division, Box<(dyn StdError + std::marker::Send + Sync + 'static)>> {
    println!("  updating thread: {:?}", division_obj.division_id);
    let full_division_obj = fetch_division(division.division_id).await.unwrap();

    ChannelId(division.discord_thread_id as u64)
        .send_message(http, |m| m.content("this is some contents"))
        .await
        .unwrap();

    // DONE

    todo!("post a message inside this thread")
}

async fn fixture_thread(
    connection: &mut PgConnection,
    http: &Arc<Http>,
    division_obj: &DivisionObj,
) -> Result<model::Division, Box<(dyn StdError + std::marker::Send + Sync + 'static)>> {
    println!("  fixturing thread: {:?}", division_obj.division_id);

    // Create the discord thread for this.
    let message = ChannelId(1092090816178171935)
        .send_message(&http, |m| m.content(division_obj.title.as_str()))
        .await
        .unwrap();

    let channel = ChannelId(1092090816178171935)
        .create_public_thread(&http, message.id, |m| m.name(division_obj.title.as_str()))
        .await
        .unwrap();

    // Save the Discord thread.
    create_division(
        connection,
        division_obj.division_id,
        *channel.id.as_u64() as i64,
    )
}

#[async_trait]
impl EventHandler for OpenGovBotHandler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        // println!("hello: {:?}", msg);

        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url}"))
}

fn create_division(
    conn: &mut PgConnection,
    division_id: i32,
    discord_thread_id: i64,
) -> Result<Division, Box<dyn Error + Send + Sync>> {
    let new_division = NewDivision {
        division_id,
        discord_thread_id,
    };

    let division = diesel::insert_into(divisions::table)
        .values(&new_division)
        .returning(Division::as_returning())
        .get_result(conn)?;

    Ok(division)
}

// fn new_division_update(
//     conn: &mut PgConnection,
//     division_id: i32,
//     publication_updated: &str,
// ) -> Result<DivisionUpdate, Box<dyn Error + Send + Sync>> {
//     let new_division_update = NewDivisionUpdate {
//         division_id,
//         publication_updated,
//     };

//     let division_update = diesel::insert_into(division_updates::table)
//         .values(&new_division_update)
//         .returning(DivisionUpdate::as_returning())
//         .get_result(conn)?;

//     Ok(division_update)
// }

use hyper::body::Buf;
use hyper_tls::HttpsConnector;

type ResultRes<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct DivisionObj {
    pub division_id: i32,
    pub title: String,
    pub publication_updated: String,
}

async fn fetch_divisions() -> ResultRes<Vec<DivisionObj>> {
    let url = "https://commonsvotes-api.parliament.uk/data/divisions.json/search"
        .parse()
        .unwrap();

    let https = HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    let res = client.get(url).await?;
    let body = hyper::body::aggregate(res).await?;
    let divisions = serde_json::from_reader(body.reader())?;

    Ok(divisions)
}

async fn fetch_division(id: i32) -> ResultRes<DivisionObj> {
    let url = format!(
        "https://commonsvotes-api.parliament.uk/data/division/{:?}.json",
        id
    )
    .parse()
    .unwrap();

    let https = HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    let res = client.get(url).await?;
    let body = hyper::body::aggregate(res).await?;
    let divisions = serde_json::from_reader(body.reader())?;
    Ok(divisions)
}
