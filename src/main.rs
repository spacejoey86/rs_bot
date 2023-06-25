use std::fs;
use std::collections::BTreeMap;

use lazy_static::lazy_static;
use std::sync::Mutex;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::GuildId;
use serenity::prelude::*;

use serde_json::Value;
use serde::{Deserialize, Serialize};

use chrono_tz::{Tz, TZ_VARIANTS};
use chrono::Utc;


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

#[derive(Serialize, Deserialize)]
struct Mapping {
    data: Vec<(GuildId, Vec<(String, String)>)>
}

lazy_static! {
    static ref TEST: Mutex<BTreeMap<GuildId, Vec<(String, String)>>> = {
        let m = BTreeMap::new();
        let mutx = Mutex::new(m);

        let zone_file = fs::read_to_string("zones.json");
        let zones: Mapping;
        match zone_file {
            Ok(zone_file) => {
                zones = match serde_json::from_str(&zone_file) {
                    Ok(r) => {r},
                    Err(_) => {
                        println!("Failed to read from zones.json");
                        // write_tzs();
                        Mapping {data : Vec::new()}
                    }
                };
            },
            Err(_) => {
                // write_tzs();
                zones = Mapping {data : Vec::new()};
                // write_tzs();
                // panic!("Could not open zones.json: {}", e)
            }
        }

        // let d: Mapping = 


        for (guild, vec) in zones.data.iter() {
            mutx.lock().unwrap().insert(*guild, vec.to_vec());
        }
        
        // m.insert(GuildId(952651810546540606), vec![("the Americans".to_string(), "US/Eastern".to_string())]);
        // m.insert(GuildId(952651810546540606), vec![("joey".to_string(), "Europe/London".to_string())]);

        // Australian Eastern Standard Time (AEST)
        mutx
    };
}

fn write_tzs() {
    let mut data: Vec<(GuildId, Vec<(String, String)>)> = Vec::new();
    // TEST.lock().unwrap().
    // let m: BTreeMap<GuildId, Vec<(String, String)>> = TEST.lock().unwrap();
    for (key, val) in TEST.lock().unwrap().iter() {
        data.push((*key, val.to_vec()))
    }

    for (guild, vec) in data.iter() {
        TEST.lock().unwrap().insert(*guild, vec.to_vec());
    }

    fs::write("zones.json", serde_json::to_string(&Mapping { data : data}).unwrap()).unwrap();
}

fn get_time_str(guild_id: GuildId) -> String {
    let now = Utc::now();
    let mut output = "".to_string();

    match TEST.lock().unwrap().get(&guild_id) {
        Some(v) => {
            for (who, timezone) in v.iter() {
                output += &("\nfor ".to_string() + who + " it is ");
                let who_time: chrono::DateTime<_> = now.with_timezone(&timezone.parse::<Tz>().unwrap());
                output += &who_time.time().format("%H:%M").to_string()
            }
            
        },
        None => {
            output += "No timezone data for this server, use /tzadd <who> <timezone>"
        }
    }

    // let usa_tz = "US/Eastern".parse::<Tz>().unwrap();
    // let usa_time = now.with_timezone(&usa_tz);
    // output += &format!("For the Americans it is currently {}", usa_time.format("%H:%M"));

    // let uk_time: chrono::DateTime<_> = now.with_timezone(&"Europe/London".parse::<Tz>().unwrap());
    // output += &format!("\nFor joey in the UK it is {}", uk_time.time().format("%H:%M"));
    // if usa_time.format("%d/%m/%y").to_string() != uk_time.format("%d/%m/%y").to_string() {
    //     output += " the next day"
    // }

    output += "\nI'm a bot, message joey if something went wrong or needs changing";
    // let aus_time = ""
    return output;
    // return format!("For the Americans it is currently {}\nFor joey in the UK it is {}", usa_time.format("%H:%M"), uk_time.time().format("%H:%M"))
    // return format!("test at guild {}", guild_id)
}


#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {

        if msg.content.starts_with("/tzadd") {
            let list: Vec<&str> = msg.content.split(" ").collect();
            if list.len() != 3 {
                if let Err(why) = msg.reply(&ctx.http, "usage: /tzadd person timezone").await {
                    println!("Error sending message: {:?}", why);
                }
                return;
            }

            if ! TZ_VARIANTS.map(|tz| tz.to_string()).contains(&list[2].to_string()) {
                if let Err(why) = msg.reply(&ctx.http, "Unknown timezone, full list comming soon").await {
                    println!("Error sending message: {:?}", why);
                }
                return;
            }

            if !TEST.lock().unwrap().contains_key(&msg.guild_id.unwrap()) {
                TEST.lock().unwrap().insert(msg.guild_id.unwrap(), Vec::new());
            }
            TEST.lock().unwrap().get_mut(&msg.guild_id.unwrap()).unwrap().push((list[1].to_string(), list[2].to_string()));
            write_tzs();
            if let Err(why) = msg.channel_id.say(&ctx.http, format!("Added {} to {}", list[1], list[2])).await {
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
    // for tz in chrono_tz::TZ_VARIANTS {
    //     println!("{}", tz);
    // }

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