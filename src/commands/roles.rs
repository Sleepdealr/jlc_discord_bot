// Future role commands will be here
use serenity::framework::standard::Args;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
    prelude::*,
};

#[command]
async fn iam(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let arg_role = args.single_quoted::<String>()?;
    if let Some(guild_id) = msg.guild_id {
        if let Some(guild) = guild_id.to_guild_cached(&ctx).await {
            if let Some(role) = guild.role_by_name(arg_role.as_str()) {
                let mut member = guild.member(ctx, msg.author.id).await?;
                member.add_role(ctx, role.id).await?;
            } else {
                msg.react(ctx, '👎').await?;
                return Ok(())
            }
        }
    }
    msg.react(ctx, '👍').await?;
    Ok(())
}

#[command]
async fn iamnot(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let arg_role = args.single_quoted::<String>()?;
    if let Some(guild_id) = msg.guild_id {
        if let Some(guild) = guild_id.to_guild_cached(&ctx).await {
            if let Some(role) = guild.role_by_name(arg_role.as_str()) {
                let mut member = guild.member(ctx, msg.author.id).await?;
                member.remove_role(ctx, role.id).await?;
            } else {
                msg.react(ctx, '👎').await?;
                return Ok(())
            }
        }
    }
    msg.react(ctx, '👍').await?;
    Ok(())
}
