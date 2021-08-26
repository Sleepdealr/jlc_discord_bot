use chrono::Utc;
use serenity::{
    framework::standard::{
        CommandResult,
        macros::command,
    },
    model::channel::Message,
    prelude::*,
};

use crate::read_json;

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let component_list = read_json("config/components.json");
    let mut name_list: String = "".to_string();
    for component in component_list.components {
        name_list.push_str(component.name.as_str());
        name_list.push_str("\n");
    }
    if let Err(why) = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("All current components");
                e.timestamp(&Utc::now());
                e.field("Components", name_list, false);
                e
            })
        })
        .await
    {
        eprintln!("Error sending message: {:?}", why);
    };
    Ok(())
}