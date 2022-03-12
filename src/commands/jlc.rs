use crate::utils::jlc::{get_components, get_datasheets};
use crate::OWNER_CHECK;
use crate::{jlc_stock_check, DatabasePool, Datasheet};
use chrono::Utc;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};
use std::sync::Arc;

#[command("list")]
#[sub_commands(components, datasheets)]
async fn list_upper(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.reply(&ctx.http, "Specify either components or datasheets")
        .await?;
    Ok(())
}

#[command]
#[aliases("c")]
#[description("Display current component list")]
async fn components(ctx: &Context, msg: &Message) -> CommandResult {
    let component_list = get_components(ctx).await; // Components list from DB

    let mut name_list: String = "".to_string(); // String with newlines to contain all Components
    for component in component_list {
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
#[aliases("d")]
#[description("Display current datasheet list")]
async fn datasheets(ctx: &Context, msg: &Message) -> CommandResult {
    let datasheet_list: Vec<Datasheet> = get_datasheets(ctx).await; // Datasheet list from DB
    let mut embed_list: String = "".to_string();

    // Format all datasheets and push onto sting for printing
    for datasheet in datasheet_list {
        embed_list.push_str(&format!(
            "[{text}]({url})\n", // Formatting for discord links is same as markdown
            text = datasheet.name,
            url = datasheet.link
        ));
    }
    if let Err(why) = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Datasheets");
                e.timestamp(&Utc::now());
                e.field("Datasheets", embed_list, false);
                e
            })
        })
        .await
    {
        eprintln!("Error sending message: {:?}", why);
    };
    Ok(())
}

#[command("add")]
#[sub_commands(add_component, add_datasheet)]
async fn add_upper(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    // This isn't supposed to be calls so just returns usage
    msg.reply(&ctx.http, "Specify either components or datasheets")
        .await?;
    Ok(())
}

#[command("add")]
#[description("Add component to the database")]
#[aliases("c")]
#[num_args(3)]
#[checks(Owner)]
async fn add_component(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Read passed arguments
    // Single quoted allows bot to read arguments with spaces when in quotes
    let name = args.single_quoted::<String>().unwrap();
    let lcsc = args.single_quoted::<String>().unwrap();
    let channel = args.single_quoted::<i64>().unwrap();

    let data_read = ctx.data.read().await;
    let pool = data_read.get::<DatabasePool>().unwrap();

    let query = format!(
        "INSERT INTO components (name, lcsc, enabled, channel_id, stock, role_id)\
    VALUES ('{}', '{}', {}, {}, {}, {})",
        name, lcsc, true, channel, 1, 0
    );
    sqlx::query(query.as_str()).execute(pool).await?;

    msg.channel_id.say(&ctx.http, "Created component").await?;

    Ok(())
}

#[command]
#[aliases("d")]
#[num_args(2)]
#[description("Add datasheet to the database")]
#[checks(Owner)]
async fn add_datasheet(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name = args.single_quoted::<String>().unwrap();
    let link = args.single_quoted::<String>().unwrap();

    let data_read = ctx.data.read().await;
    let pool = data_read.get::<DatabasePool>().unwrap();

    let query = format!(
        "INSERT INTO datasheets (name, link) VALUES ('{}' , '{}')",
        name, link
    );
    sqlx::query(query.as_str()).execute(pool).await?;

    msg.channel_id.say(&ctx.http, "Added datasheet").await?;
    Ok(())
}

#[command]
#[description("Disable a component")]
#[num_args(1)]
#[checks(Owner)]
async fn disable(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let lcsc = args.single_quoted::<String>().unwrap();

    let data_read = ctx.data.read().await;
    let pool = data_read.get::<DatabasePool>().unwrap();

    let statement = format!("SELECT enabled FROM components WHERE lcsc = '{}'", lcsc);
    let select: (bool,) = sqlx::query_as(statement.as_str()).fetch_one(pool).await?;
    let enabled = !select.0;

    let update = format!(
        "UPDATE components SET enabled={} WHERE lcsc='{}'",
        enabled, lcsc
    );

    sqlx::query(update.as_str()).execute(pool).await?;

    msg.channel_id.say(&ctx.http, "Updated component").await?;

    Ok(())
}

#[command]
#[checks(Owner)]
#[description("Manually trigger JLC stock check")]
async fn check(ctx: &Context, _msg: &Message) -> CommandResult {
    let arc = ctx.clone();
    jlc_stock_check(Arc::new(arc)).await;
    Ok(())
}
