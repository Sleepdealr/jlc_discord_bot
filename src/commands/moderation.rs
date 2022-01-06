use serenity::framework::standard::macros::command;
use serenity::{
    framework::standard::{Args, CommandError, CommandResult},
    model::prelude::*,
    prelude::*,
    Error,
};

use crate::utils::users::get_members;

#[command]
#[required_permissions(MANAGE_MESSAGES)]
#[num_args(1)]
#[description("Deletes messages. (limit 99 at a time)")]
async fn clear(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let num = args.single::<u64>();
    match num {
        Err(_) => {
            msg.channel_id
                .say(ctx, "Value provided was not a number.")
                .await?;
        }
        Ok(n) => {
            let channel = &msg.channel(ctx).await.unwrap().guild().unwrap();

            let messages = &channel
                .messages(ctx, |r| r.before(&msg.id).limit(n))
                .await?;
            let messages_ids = messages.iter().map(|m| m.id).collect::<Vec<MessageId>>();

            match channel.delete_messages(ctx, messages_ids).await {
                Ok(()) => {
                    let returnmsg = msg
                        .channel_id
                        .say(ctx, format!("deleted `{}` messages", n))
                        .await?;
                    msg.delete(ctx).await?;
                    returnmsg.delete(ctx).await?;
                }
                Err(Error::Model(ModelError::InvalidPermissions(permissions))) => {
                    println!("{:?}", permissions);
                    return Err(CommandError::from("Missing Permissions: MANAGE_MESSAGES"));
                }
                Err(e) => {
                    println!("{:?}", e);
                    return Err(CommandError::from("Missing Permissions: MANAGE_MESSAGES"));
                }
            }
        }
    }
    Ok(())
}

#[command]
#[description("Bans people. (limit one at a time)")]
async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let to_parse = args.single_quoted::<String>()?;
    let member = get_members(ctx, msg, to_parse).await;

    let reason = args.remains();

    match member {
        Ok(m) => {
            if let Some(r) = reason {
                m.ban_with_reason(ctx, 0, r).await?;
                msg.channel_id
                    .say(ctx, format!("banned `{}` because `{}`", m.user.tag(), r))
                    .await?;
            } else {
                m.ban(ctx, 0).await?;
                msg.channel_id
                    .say(ctx, format!("banned `{}`, no reason given.", m.user.tag()))
                    .await?;
            }
        }
        Err(why) => return Err(CommandError::from(why.to_string())),
    }

    Ok(())
}

#[command]
#[required_permissions(KICK_MEMBERS)]
#[description("Kicks people. (limit one at a time)")]
async fn kick(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let to_parse = args.single_quoted::<String>()?;
    let member = get_members(ctx, msg, to_parse).await;

    let reason = args.remains();

    match member {
        Ok(m) => {
            if let Some(r) = reason {
                m.kick_with_reason(ctx, r).await?;
                msg.channel_id
                    .say(ctx, format!("kicked `{}` because `{}`", m.user.tag(), r))
                    .await?;
            } else {
                m.kick(ctx).await?;
                msg.channel_id
                    .say(ctx, format!("kicked `{}`, no reason given.", m.user.tag()))
                    .await?;
            }
        }
        Err(why) => return Err(CommandError::from(why.to_string())),
    }

    Ok(())
}
