use crate::keys::BotCtl;
use crate::Uptime;
use chrono::Utc;
use heim::{memory, process, units};
use serenity::framework::standard::CommandOptions;
use serenity::framework::standard::Reason;
use serenity::{
    framework::standard::{
        macros::{check, command},
        Args, CommandResult,
    },
    model::channel::Message,
    prelude::*,
};
use std::sync::atomic::Ordering;
use timeago;
use tokio::time;

#[check]
#[name = "Owner"]
pub async fn owner_check(
    _: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    if msg.author.id != 236222353405640704 {
        let result = Err(Reason::User("Lacked owner permission".to_string()));
        return result;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(Owner)]
#[description = "Toggle bot loop"]
async fn toggle_bot(ctx: &Context, msg: &Message) -> CommandResult {
    let data_read = ctx.data.read().await;
    let value = data_read
        .get::<BotCtl>()
        .expect("Expected MessageCount in TypeMap.")
        .clone();
    value.store(!value.load(Ordering::Relaxed), Ordering::Relaxed);
    msg.channel_id.say(&ctx.http, "Bot toggled").await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[description = "Print bot stats"]
async fn stats(ctx: &Context, msg: &Message) -> CommandResult {
    println!("Stats!");
    let bot_version = env!("CARGO_PKG_VERSION");

    // Use heim to get resources
    let memory = memory::memory().await.unwrap();
    let process = process::current().await.unwrap();
    let thismem = process.memory().await.unwrap();
    let fullmem = memory.total();
    let cpu_1 = process.cpu_usage().await.unwrap();

    time::sleep(time::Duration::from_millis(100)).await;

    let cpu_2 = process.cpu_usage().await.unwrap();

    let uptime = {
        let data = ctx.data.read().await;
        match data.get::<Uptime>() {
            Some(time) => {
                if let Some(boot_time) = time.get("boot") {
                    let now = Utc::now();
                    let mut f = timeago::Formatter::new();
                    f.num_items(4);
                    f.ago("");

                    f.convert_chrono(boot_time.to_owned(), now)
                } else {
                    "Uptime not available".to_owned()
                }
            }
            None => "Uptime not available.".to_owned(),
        }
    };

    let mut f = timeago::Formatter::new();
    f.num_items(4);
    f.ago("");

    let shard_plural = if ctx.cache.shard_count().await > 1 {
        "s"
    } else {
        ""
    };
    let avatar = ctx
        .cache
        .current_user()
        .await
        .avatar_url()
        .unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_string());
    let shards = ctx.cache.shard_count().await;

    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(0x3498db)
                    .title(&format!("jlc_discord_bot v{}", bot_version))
                    .thumbnail(&format!("{}", avatar))
                    .field("Author", "Sleepdealer#0001", false)
                    .field(
                        "Memory",
                        format!(
                            "`{} MB used`\n`{} MB virt`\n`{} GB available`",
                            &thismem.rss().get::<units::information::megabyte>(),
                            &thismem.vms().get::<units::information::megabyte>(),
                            &fullmem.get::<units::information::gigabyte>()
                        ),
                        true,
                    )
                    .field(
                        "CPU",
                        format!("`{}%`", (cpu_2 - cpu_1).get::<units::ratio::percent>()),
                        true,
                    )
                    .field(
                        "Shards",
                        format!("`{} shard{}` ", shards, shard_plural),
                        true,
                    )
                    .field("Bot Uptime", &uptime, false);
                e
            });
            m
        })
        .await;

    Ok(())
}

#[command]
#[checks(Owner)]
async fn crashandburn(_ctx: &Context, _msg: &Message) -> CommandResult {
    std::process::exit(1)
}
