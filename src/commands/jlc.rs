use crate::utils::jlc::{get_components, read_datasheet_json};
use crate::{jlc_stock_check, DatabasePool, Datasheet};
use chrono::Utc;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};
use std::sync::Arc;

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let component_list = get_components(ctx).await;

    let mut name_list: String = "".to_string();
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
async fn add_component(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name = args.single_quoted::<String>().unwrap();
    let lcsc  = args.single_quoted::<String>().unwrap();
    let channel = args.single_quoted::<i64>().unwrap();

    let data_read = ctx.data.read().await;
    let pool = data_read.get::<DatabasePool>().unwrap();

    sqlx::query!(
        r#"
        INSERT INTO components (name, lcsc, enabled, channel_id, stock, role_id)
        VALUES ($1 , $2 , $3 , $4 , $5 , $6)
        "#,
        name, lcsc, true, channel, 1, 0
    )
    .execute(pool)
    .await?;

    msg.channel_id.say(&ctx.http, "Created component").await?;

    Ok(())
}

#[command]
async fn disable(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let arg_vec: Vec<_> = args.rest().split_whitespace().collect();
    let lcsc = arg_vec[0].to_string();

    let data_read = ctx.data.read().await;
    let pool = data_read.get::<DatabasePool>().unwrap();

    let statement = format!("SELECT enabled FROM components WHERE lcsc = '{}'" , lcsc);
    let select:(bool,) = sqlx::query_as(statement.as_str())
        .fetch_one(pool)
        .await?;
    let enabled = !select.0;


    let update = format!("UPDATE components SET enabled={} WHERE lcsc='{}'" , enabled , lcsc);

    sqlx::query(update.as_str())
        .execute(pool)
        .await?;

    msg.channel_id.say(&ctx.http, "Updated component").await?;

    Ok(())
}

#[command]
async fn check_jlc(ctx: &Context, _msg: &Message) -> CommandResult {
    let arc = ctx.clone();
    jlc_stock_check(Arc::new(arc)).await;
    Ok(())
}

#[command]
async fn datasheets(ctx: &Context, msg: &Message) -> CommandResult {
    let datasheet_list: Vec<Datasheet> = read_datasheet_json(ctx).await;
    let mut embed_list: String = "".to_string();
    for datasheet in datasheet_list {
        embed_list.push_str(&format!(
            "[{text}]({url})\n",
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

#[command]
async fn add_datasheet(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let arg_vec: Vec<_> = args.rest().split_whitespace().collect();
    let name = arg_vec[0].to_string();
    let link = arg_vec[1].to_string();

    let data_read = ctx.data.read().await;
    let pool = data_read.get::<DatabasePool>().unwrap();

    sqlx::query!(
        r#"
        INSERT INTO datasheets (name, link)
        VALUES ($1 , $2)
        "#,
        name,
        link
    )
    .execute(pool)
    .await?;

    msg.channel_id.say(&ctx.http, "Added datasheet").await?;
    Ok(())
}
