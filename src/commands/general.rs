// General purpose commands

use serenity::{
    framework::standard::{Args, CommandResult, macros::command},
    model::channel::Message,
    prelude::*,
    utils::{content_safe, ContentSafeOptions},
};

#[command]
async fn echo(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let settings = if let Some(guild_id) = msg.guild_id {
        ContentSafeOptions::default()
            .clean_channel(false)
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
    };

    let content = content_safe(&ctx.cache, &args.rest(), &settings).await;

    msg.channel_id.say(&ctx.http, &content).await?;

    Ok(())
}
