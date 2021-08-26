use std::sync::atomic::Ordering;

use serenity::{
    framework::standard::{
        Args,
        CommandResult, macros::{check, command},
    },
    model::channel::Message,
    prelude::*,
};
use serenity::framework::standard::CommandOptions;
use serenity::framework::standard::Reason;

// Commands about and to control the bot will be here
use crate::keys::BotCtl;

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
