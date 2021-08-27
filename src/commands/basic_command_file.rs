use serenity::{
    framework::standard::{Args, CommandResult, macros::command},
    model::channel::Message,
    prelude::Context,
};

#[command]
async fn basic_command(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.channel_id.say(ctx, "Hello World!").await?;
    Ok(())
}
