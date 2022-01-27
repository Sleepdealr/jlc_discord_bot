use std::sync::atomic::Ordering;

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
use timeago;
use tokio::time;

// Commands about and to control the bot will be here
use crate::keys::BotCtl;
use crate::utils::general::get_uptime;

#[check]
#[name = "Owner"]
async fn owner_check(
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
async fn stats(ctx: &Context, msg: &Message) -> CommandResult {
    println!("Stats!");
    let bot_version = env!("CARGO_PKG_VERSION");

    let memory = memory::memory().await.unwrap();
    let process = process::current().await.unwrap();
    let thismem = process.memory().await.unwrap();
    let fullmem = memory.total();
    let cpu_1 = process.cpu_usage().await.unwrap();

    time::sleep(time::Duration::from_millis(100)).await;

    let cpu_2 = process.cpu_usage().await.unwrap();

    let uptime = get_uptime(&ctx).await;

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
    std::process::exit(0)
}
