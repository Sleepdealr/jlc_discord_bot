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


use sys_info::mem_info;
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
use chrono::offset::Utc;
async fn set_status_to_current_time(ctx: Arc<Context>) {
    let current_time = Utc::now();
    let formatted_time = current_time.to_rfc2822();

    ctx.set_activity(Activity::playing(&formatted_time)).await;
}