use crate::Component;
use chrono::Utc;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};
use std::fs::File;

use crate::utils::jlc::read_components_json;

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let component_list = read_components_json("config/components.json");
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

#[command]
async fn add_component(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let arg_vec: Vec<_> = args.rest().split_whitespace().collect();
    let product_name = arg_vec[0].to_string();
    let lcsc_number = arg_vec[1].to_string();
    let channel = arg_vec[2].parse::<u64>()?;

    let new_component = Component {
        name: product_name,
        lcsc: lcsc_number,
        enabled: true,
        channel_id: channel,
        prev_stock: 1,
        role_id: 0,
    };

    let mut component_list = read_components_json("config/components.json");
    component_list.components.push(new_component);

    serde_json::to_writer_pretty(
        &File::create("config/components.json").expect("File creation error"),
        &component_list,
    )
    .expect("Error writing file");
    msg.channel_id.say(&ctx.http, "Created component").await?;
    Ok(())
}
