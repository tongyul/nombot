use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod commands;
mod defn;
mod nom_args;
mod nom_util;

use crate::defn::command::{ ClientData, CommandHandler };
use crate::defn::globals::{ CommandMap, CommandMapTmk };

struct Handler {
    command_prefix: String,
}

impl Handler {
    fn new(command_prefix: String) -> Self { Self { command_prefix } }
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message is received - the
    // closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be dispatched
    // simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        let pref = &self.command_prefix[..];
        if msg.content.len() >= pref.len() && &msg.content.chars().take(pref.chars().count()).collect::<String>()[..] == pref {
            // Sending a message can fail, due to a network error, an authentication error, or lack
            // of permissions to post in the channel, so log to stdout when some error happens,
            // with a description of it.
            let content_tail = &msg.content[pref.len()..];
            match nom_args::parse(content_tail) {
                Err(e) => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, format!("```\n{}\n```", e)).await {
                        println!("Error sending message: {why:?}");
                    }
                }
                Ok(cmd) => {
                    let h = {
                        let data = ctx.data.read().await;
                        let cm = data
                            .get::<CommandMapTmk>().expect("Command map does not exist!")
                            .read().await;
                        cm.get(&cmd.name[..]).map(Arc::clone)
                    };
                    match h {
                        None => {
                            if let Err(why) = msg.channel_id.say(&ctx.http, format!("```\nCommand {:?} does not exist\n```", cmd.name)).await {
                                println!("Error sending message: {why:?}");
                            }
                        }
                        Some(h) => h.call(cmd, ctx, msg).await,
                    }
                }
            };
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN in the environment.");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut prefix = env::var("PREFIX").expect("Expected a PREFIX in the environment.");
    prefix.push('/');
    let handler = Handler::new(prefix);
    let mut client =
        Client::builder(&token, intents).event_handler(handler).await.expect("Err creating client");
    let command_map = Arc::new(RwLock::new(HashMap::new()));

    async fn register<H: 'static + CommandHandler>(mut h: H, d: ClientData, c: CommandMap) {
        let keys = h.register(d).await;
        let mut c = c.write().await;
        let h_arc = Arc::new(h);
        for k in keys.into_iter() {
            if let Some(_) = c.insert(k, h_arc.clone()) {
                panic!("The same name is bound to multiple commands");
            }
        }
    }

    tokio::join!(
        register(commands::echo::EchoHandler, client.data.clone(), command_map.clone()),
        register(commands::help::HelpHandler, client.data.clone(), command_map.clone()),
        register(commands::nom::NomHandler, client.data.clone(), command_map.clone()),
    );
    {
        let mut data = client.data.write().await;
        data.insert::<CommandMapTmk>(command_map);
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
