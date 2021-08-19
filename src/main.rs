// TODO: Basic functionality
//  - Read stock data from JLC site - COMPLETE
//  - Iterate over all needed components
//  - Send this data to component's channels
//  - Ping roles if components is back in stock
//  - Set status to time until next update
//
// TODO: Command utilities
//  - Add command to toggle specific components/everything
//  - Add removal/addition of components with commands
//
// TODO: Implement JSON
//  - Use JSON or other file format to contain components/channels/roles/previous stock
//  - If previous component stock was 0, ping role with message
//
// TODO: Logs (Future)
//  - Log lifetime usages?


use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc
    },
    time::Duration
};

use chrono::offset::Utc;
use serenity::{
    async_trait,
    client::bridge::gateway::{ShardManager},
    framework::standard::{
        help_commands,
        macros::{check, command, group, help, hook},
        Args,
        CommandGroup,
        CommandOptions,
        CommandResult,
        DispatchError,
        HelpOptions,
        Reason,
        StandardFramework,
    },
    http::Http,
    model::{
        channel::{Message},
        gateway::{Activity , Ready},
        id::{UserId , ChannelId , GuildId},

    },
    utils::{content_safe, ContentSafeOptions},
    prelude::*,

};

use tokio::sync::Mutex;

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

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
            if self.enable_bot {
                tokio::spawn(async move {
                    loop {
                        print_stock_data(Arc::clone(&ctx1)).await.expect("Error printing stock data");
                        println!("Sent stock data");
                        tokio::time::sleep(Duration::from_secs(120)).await;
                    }
                });
            }

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
#[commands(echo, commands)]
struct General;

#[group]
#[owners_only]
#[only_in(guilds)]
#[summary = "Commands for server owners"]
#[commands(ping)]
struct Owner;

#[help]
#[individual_command_tip = "Hello! This is a JLCPCB component stock checker bot\n\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find command: `{}`."]
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible
// cases of ~~strikethrough-commands~~, but only if
// `strikethrough_commands_tip_in_{dm, guild}` aren't specified.

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
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Got command '{}' by user '{}'", command_name, msg.author.name);

    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
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
    let unknown = format!("Could not find command named '{}'" , unknown_command_name);
    println!("{}" , unknown);
    _msg.reply(&_ctx.http , unknown).await.expect("Error replying to message");

}

#[hook]
async fn normal_message(_ctx: &Context, _msg: &Message)  {
    // println!("Message is not a command '{}'", msg.content);
}

#[hook]
async fn delay_action(ctx: &Context, msg: &Message) {
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                .await;
        }
    }
}

async fn get_jlc_stock(lcsc: &str) -> Result<i64, reqwest::Error> {
    let echo_json: serde_json::Value = reqwest::Client::new()
        .post("https://jlcpcb.com/shoppingCart/smtGood/selectSmtComponentList")
        .json(&serde_json::json!({
            "keyword": lcsc
        }))
        .send()
        .await?
        .json()
        .await?;

    let stock = echo_json["data"]["componentPageInfo"]["list"][0]["stockCount"].as_i64().unwrap();
    Ok(stock)
}

async fn print_stock_data(ctx: Arc<Context>) -> CommandResult {
    let stock = get_jlc_stock("C112161").await.expect("Error");
    if let Err(why) = ChannelId(877779984448638976)
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Atmega32u4-MU");
                e.field(
                    "Stock",
                    format!(
                        "{}",
                        stock
                    ),
                    false,
                );
                e
            })
        })
        .await
    {
        eprintln!("Error sending message: {:?}", why);
    };
    Ok(())
}

#[tokio::main]
async fn main() {
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
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix("!")
            .delimiters(vec![", ", ","])
            .owners(owners)
        )
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
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
#[bucket = "complicated"]
async fn commands(ctx: &Context, msg: &Message) -> CommandResult {
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.read().await;
    let counter = data.get::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");

    for (k, v) in counter {
        writeln!(contents, "- {name}: {amount}", name = k, amount = v)?;
    }

    msg.channel_id.say(&ctx.http, &contents).await?;

    Ok(())
}

#[command]
async fn echo(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let settings = if let Some(guild_id) = msg.guild_id {
        ContentSafeOptions::default()
            .clean_channel(false)
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default().clean_channel(false).clean_role(false)
    };

    let content = content_safe(&ctx.cache, &args.rest(), &settings).await;

    msg.channel_id.say(&ctx.http, &content).await?;

    Ok(())
}


#[check]
#[name = "Owner"]
async fn owner_check(
    _: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {

    if msg.author.id != 236222353405640704 {
        return Err(Reason::User("Lacked owner permission".to_string()));
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(Owner)]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong! : )").await?;

    Ok(())
}

#[command]
async fn toggle_bot(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    msg.channel_id.say(&ctx.http, "Pong! : )").await?;

    Ok(())
}