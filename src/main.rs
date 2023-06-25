use std::fs;
use std::collections::BTreeMap;
use std::sync::Arc;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{GuildId};
use serenity::prelude::*;

use serde_json::Value;

use chrono_tz::Tz;
use chrono::{TimeZone, Utc, DateTime};


struct Handler;

async fn get_conf() -> Value {
    let config_file = fs::read_to_string("config.json");
    match config_file {
        Ok(config) => {
            let conf: Value = serde_json::from_str(&config).unwrap();
            return conf;
        },
        Err(e) => {
            panic!("Could not open config.json: {}", e)
        }
    }
}

trait FromVal {
    fn get_config(key: &str, conf: Value) -> Self
    where
        Self: Sized;
}

impl FromVal for String {
    fn get_config(key: &str, conf: Value) -> Self
        where
            Self: Sized {
        match &conf[key] {
            Value::String(s) => {
                return s.to_string();
            },
            _ => {
                panic!("Malformed config file, see example_config.json for an example")
            }
        }
    }
}

// static mut mapping: Arc<i32> = Arc::new(0);

fn get_time_str(_guild_id: GuildId) -> String {
    let now = Utc::now();
    let mut output = "".to_string();


    let usa_tz = "US/Eastern".parse::<Tz>().unwrap();
    let usa_time = now.with_timezone(&usa_tz);
    output += &format!("For the Americans it is currently {}", usa_time.format("%H:%M"));

    let uk_time: chrono::DateTime<_> = now.with_timezone(&"Europe/London".parse::<Tz>().unwrap());
    output += &format!("\nFor joey in the UK it is {}", uk_time.time().format("%H:%M"));
    if usa_time.format("%d/%m/%y").to_string() != uk_time.format("%d/%m/%y").to_string() {
        output += " the next day"
    }

    // let aus_time = ""
    return output;
    // return format!("For the Americans it is currently {}\nFor joey in the UK it is {}", usa_time.format("%H:%M"), uk_time.time().format("%H:%M"))
    // return format!("test at guild {}", guild_id)
}


#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
        else if ! msg.author.bot &&
                msg.content.to_ascii_lowercase().contains("time") &&
                msg.content.to_ascii_lowercase().contains("zone") {
                    match msg.guild_id {
                        Some(gid) => {
                            if let Err(why) = msg.channel_id.say(&ctx.http, get_time_str(gid)).await {
                                println!("Error sending message: {:?}", why);
                            }
                        },
                        None => {
                            // no guild ID, do nothing
                        }
                    }

        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let conf = get_conf();
    let token = String::get_config(&"APIKey", conf.await);
    
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;


    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}