// TODO: Basic functionality - COMPLETE
//  - Read stock data from JLC site - COMPLETE
//  - Iterate over all needed components - COMPLETE
//  - Send this data to component's channels - COMPLETE
//  - Ping roles if components is back in stock - COMPLETE
//  - Add more info to embed - COMPLETE
//      - Link to product page - COMPLETE
//      - Image in Embed - COMPLETE
//
// TODO - Time utilities
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

use std::{
    collections::HashSet,
    env, fs,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use std::fs::File;

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

use commands::general::*;
use commands::jlc::*;
use commands::meta::*;
use keys::*;

use crate::utils::jlc::print_stock_data;

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
                        let mut component_list: Components = read_json("config/components.json");
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
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(echo, list)]
struct General;

#[group]
#[owners_only]
#[only_in(guilds)]
#[summary = "Commands for server owners"]
#[commands(toggle_bot)]
struct Owner;

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

fn read_json(path: &str) -> Components {
    let file = fs::File::open(path).expect("file should open read only");
    serde_json::from_reader(file).expect("file should be proper JSON")
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
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
                .prefix("!")
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
        .group(&OWNER_GROUP);

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
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}


