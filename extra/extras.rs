
// This is just extra commands/components for possible future use

use chrono::offset::Utc;
use sys_info::mem_info;

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
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "This is a JLCPCB component tracking bot").await?;

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
async fn set_slow_mode(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let say_content = if let Ok(slow_mode_rate_seconds) = args.single::<u64>() {
        if let Err(why) =
        msg.channel_id.edit(&ctx.http, |c| c.slow_mode_rate(slow_mode_rate_seconds)).await
        {
            println!("Error setting channel's slow mode rate: {:?}", why);

            format!("Failed to set slow mode to `{}` seconds.", slow_mode_rate_seconds)
        } else {
            format!("Successfully set slow mode rate to `{}` seconds.", slow_mode_rate_seconds)
        }
    } else if let Some(Channel::Guild(channel)) = msg.channel_id.to_channel_cached(&ctx.cache).await
    {
        format!("Current slow mode rate is `{}` seconds.", channel.slow_mode_rate.unwrap_or(0))
    } else {
        "Failed to find channel in cache.".to_string()
    };

    msg.channel_id.say(&ctx.http, say_content).await?;

    Ok(())
}


async fn log_system_load(ctx: Arc<Context>) -> CommandResult {
    let cpu_load = sys_info::loadavg().unwrap();
    let mem_use = sys_info::mem_info().unwrap();

    // We can use ChannelId directly to send a message to a specific channel; in this case, the
    // message would be sent to the #testing channel on the discord server.
    if let Err(why) = ChannelId(718573098877583422)
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("System Resource Load");
                e.field("CPU Load Average", format!("{:.2}%", cpu_load.one * 10.0), false);
                e.field(
                    "Memory Usage",
                    format!(
                        "{:.2} MB Free out of {:.2} MB",
                        mem_use.free as f32 / 1000.0,
                        mem_use.total as f32 / 1000.0
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

async fn set_status_to_current_time(ctx: Arc<Context>) {
    let current_time = Utc::now();
    let formatted_time = current_time.to_rfc2822();

    ctx.set_activity(Activity::playing(&formatted_time)).await;
}


#[command]
#[description("Bot stats")]
async fn stats(ctx: &Context, msg: &Message) -> CommandResult {

    let bot_version = env!("CARGO_PKG_VERSION");

    let memory = memory::memory().await.unwrap();
    // get current process
    let process = process::current().await.unwrap();
    // get current ram
    let thismem = process.memory().await.unwrap();
    let fullmem = memory.total();
    // get current cpu
    let cpu_1 = process.cpu_usage().await.unwrap();

    time::sleep(time::Duration::from_millis(100)).await;

    let cpu_2 = process.cpu_usage().await.unwrap();

    let git_stdout;
    git_stdout = Command::new("sh")
        .arg("-c")
        .arg("git log -1 | grep ^commit | awk '{print $2}'")
        .output()
        .await.unwrap();

    let mut git_commit: String = "".to_string();

    if std::str::from_utf8(&git_stdout.stdout).unwrap() != "" {
        git_commit.push('#');
        git_commit.push_str(std::str::from_utf8(&git_stdout.stdout).unwrap());
    } else {
        git_commit.push_str("prod")
    }
    git_commit.truncate(7);


    let (name, discriminator) = match ctx.http.get_current_application_info().await {
        Ok(info) => (info.owner.name, info.owner.discriminator),
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let owner_tag = name.to_string() + "#" + &discriminator.to_string();

    let guilds_count = &ctx.cache.guilds().await.len();
    let channels_count = &ctx.cache.guild_channel_count().await;
    let users_count = ctx.cache.user_count().await;
    let users_count_unknown = ctx.cache.unknown_members().await as usize;

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

    let shard_plural = if ctx.cache.shard_count().await > 1 { "s" } else { "" };
    let avatar = ctx.cache.current_user().await.avatar_url().unwrap_or("https://cdn.discordapp.com/embed/avatars/0.png".to_string());
    let shards = ctx.cache.shard_count().await;

    let _ = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(0x3498db)
                    .title(&format!("maki v{} {}", bot_version, git_commit,))
                    .url("https://maki.iscute.dev")
                    .thumbnail(&format!("{}", avatar))
                    .field("Author", &owner_tag, false)
                    .field("Guilds", &guilds_count.to_string(), true)
                    .field("Channels", &channels_count.to_string(), true)
                    .field(
                        "Users",
                        &format!(
                            "`{} Total`\n`{} Cached`",
                            &users_count + &users_count_unknown,
                            users_count
                        ),
                        true,
                    )
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

// Old scheduling loop
//
// let data_read = ctx.data.read().await;
// if data_read
// .get::<BotCtl>()
// .expect("Expected bot toggle")
// .load(Ordering::Relaxed)
// {
// let mut component_list: Components = read_components_json("config/components.json");
// for component in &mut component_list.components {
// if component.enabled {
// let data = print_stock_data(Arc::clone(&ctx1), component).await;
// println!("Sent stock for {}", component.name);
// component.prev_stock = data;
// }
// }
// serde_json::to_writer_pretty(
// &File::create("config/components.json").expect("File creation error"),
// &component_list,
// )
// .expect("Error writing file");
// tokio::time::sleep(Duration::from_secs(86400)).await;
// }