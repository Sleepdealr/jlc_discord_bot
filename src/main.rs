// TODO: Basic functionality - COMPLETE
//  - Read stock data from JLC site - COMPLETE
//  - Iterate over all needed components - COMPLETE
//  - Send this data to component's channels - COMPLETE
//  - Ping roles if components is back in stock - COMPLETE
//  - Add more info to embed - COMPLETE
//      - Link to product page - COMPLETE
//      - Image in Embed - COMPLETE
//
// TODO - Time utilities - Probably not going to do this since it's so goddamn annoying
//  - Set bot to execute at a specific time
//  - Set status to time until next update
//
// TODO: Command utilities
//  - Add command to toggle everything - COMPLETE
//  - Add command to toggle specific components
//  - Add removal/addition of components with commands
//
// TODO: Implement JSON - COMPLETE
//  - Use JSON or other file format to contain components/channels/roles/previous stock - COMPLETE
//  - If previous component stock was 0, ping role with message - COMPLETE
//  - If previous component stock has not changed, do not send a message - COMPLETE

#[macro_use] extern crate log;
extern crate pretty_env_logger;

use std::{
    collections::HashSet,
    env,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use std::collections::HashMap;
use std::fs::File;

use chrono::Utc;
use serenity::{
    async_trait,
    framework::standard::{
        Args,
        CommandGroup,
        CommandResult, DispatchError, help_commands, HelpOptions, macros::{group, help, hook},
        StandardFramework,
    },
    http::Http,
    model::{
        channel::Message,
        gateway::Ready,
        id::{GuildId, UserId},
    },
    prelude::*,
};
use serenity::model::event::ResumedEvent;

use commands::general::*;
use commands::jlc::*;
use commands::meta::*;
use commands::moderation::*;
use commands::roles::*;
use keys::*;

use crate::utils::jlc::{print_stock_data, read_components_json};


#[macro_use]
mod utils;
mod commands;

pub mod keys;

struct Handler {
    is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        println!("Cache built successfully!");
        let ctx = Arc::new(ctx);
        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    let data_read = ctx.data.read().await;
                    if data_read
                        .get::<BotCtl>()
                        .expect("Expected bot toggle")
                        .load(Ordering::Relaxed)
                    {
                        let mut component_list: Components = read_components_json("config/components.json");
                        for component in &mut component_list.components {
                            if component.enabled {
                                let data = print_stock_data(Arc::clone(&ctx1), component).await;
                                println!("Sent stock for {}", component.name);
                                component.prev_stock = data;
                            }
                        }
                        serde_json::to_writer_pretty(
                            &File::create("config/components.json").expect("File creation error"),
                            &component_list,
                        )
                        .expect("Error writing file");
                        tokio::time::sleep(Duration::from_secs(86400)).await;
                    }
                }
            });

            // FIXME: SET STATUS HERE
            // let ctx2 = Arc::clone(&ctx);
            // tokio::spawn(async move {
            //     loop {
            //         // set_status_to_current_time(Arc::clone(&ctx2)).await;
            //         // println!("Updated status");
            //         // tokio::time::sleep(Duration::from_secs(60)).await;
            //
            //     }
            // });

            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        if let Some(shard) = ready.shard {
            info!(
                "Connected as {} on shard {}/{}",
                ready.user.name,
                shard[0] + 1,
                shard[1]
            );
        } else {
            info!("Connected as {}", ready.user.name);
        }

        let data = ctx.data.write();
        match data.await.get_mut::<Uptime>() {
            Some(uptime) => {
                uptime.entry(String::from("boot")).or_insert_with(Utc::now);
            }
            None => error!("Unable to insert boot time into client data."),
        };
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(echo, list , stats, iam)]
struct General;

#[group]
#[owners_only]
#[only_in(guilds)]
#[summary = "Commands for server owners"]
#[commands(toggle_bot)]
struct Owner;

#[group]
#[commands(clear, kick, ban)]
#[description = "Server management."]
struct Moderation;


#[help]
#[individual_command_tip = "Hello! This is a JLCPCB component stock checker bot\n\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find command: `{}`."]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]

async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    true
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    let unknown = format!("Could not find command named '{}'", unknown_command_name);
    println!("{}", unknown);
    _msg.reply(&_ctx.http, unknown)
        .await
        .expect("Error replying to message");
}

#[hook]
async fn normal_message(_ctx: &Context, _msg: &Message) {
    // println!("Message is not a command '{}'", msg.content);
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again in {} seconds.", info.as_secs()),
                )
                .await;
        }
    }
}



#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    let token = &env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let prefix = &env::var("PREFIX").expect("Expected a prefix in the environment.");
    let http = Http::new_with_token(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix(prefix)
                .delimiters(vec![", ", ","])
                .owners(owners)
        })
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&OWNER_GROUP)
        .group(&MODERATION_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
        })
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<BotCtl>(AtomicBool::new(true));
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<keys::Uptime>(HashMap::default());
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}


