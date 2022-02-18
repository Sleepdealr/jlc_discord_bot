use crate::keys::Component;
use chrono::Utc;
use futures::future::join_all;
use serde_json::Value;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use std::sync::Arc;

use crate::{DatabasePool, Datasheet};

enum JLCRequestErr {
    ReqwestError(reqwest::Error),
    BadUnwrapErr(String)
}

pub async fn get_components(ctx: &Context) -> Vec<Component> {
    // Read database pool from ctx and run a simple select * for components
    let data_read = ctx.data.read().await;
    let pool = data_read.get::<DatabasePool>().unwrap();

    let select = sqlx::query_as::<_, Component>("SELECT * FROM components");
    let components: Vec<Component> = select.fetch_all(pool).await.unwrap();

    components
}

pub async fn read_datasheet_json(ctx: &Context) -> Vec<Datasheet> {
    // Read database pool from ctx and run a simple select * for datasheets
    let data_read = ctx.data.read().await;
    let pool = data_read.get::<DatabasePool>().unwrap();

    let select = sqlx::query_as::<_, Datasheet>("SELECT * FROM datasheets");
    let datasheets: Vec<Datasheet> = select.fetch_all(pool).await.unwrap();

    datasheets
}

pub async fn get_jlc_stock(lcsc: &str) -> Result<(i64, String, f64, String), JLCRequestErr> {
    // Ping API for component and parse into serde JSON
    let response: Value = reqwest::Client::new()
        .post("https://jlcpcb.com/shoppingCart/smtGood/selectSmtComponentList")
        .json(&serde_json::json!({ "keyword": lcsc }))
        .send()
        .await
        .map_err(JLCRequestErr::ReqwestError)?
        .json()
        .await
        .map_err(JLCRequestErr::ReqwestError)?;

    let jlc_stock = response["data"]["componentPageInfo"]["list"][0]["stockCount"]
        .as_i64()
        .unwrap();

    // Error handling on image isn't necessary since it should have one if it has stock ??
    let image_url = response["data"]["componentPageInfo"]["list"][0]["componentImageUrl"]
        .as_str()
        .unwrap()
        .to_string();

    let price = response["data"]["componentPageInfo"]["list"][0]["componentPrices"][0]
        ["productPrice"]
        .as_f64()
        .unwrap();

    // JLC has basic as "base" for some reason
    let basic = match response["data"]["componentPageInfo"]["list"][0]["componentLibraryType"]
        .as_str()
        .unwrap()
    {
        "base" => "(Basic)",
        "expand" => "(Extended)",
        _ => "ERROR",
    }
    .to_string();

    // Return all values as tuple instead of list because it's easier
    // If I wanted to use a vec I would need to create an enum with all the possible types which is inconvenient for just one function
    Ok((jlc_stock, image_url, price, basic))
}

pub async fn print_stock_data(
    ctx: Arc<Context>,
    component: Component,
) -> Result<Component, JLCRequestErr> {
    let request = get_jlc_stock(&component.lcsc).await;
    // Error returned if unwrapped on a bad value
    let data = match request {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    // Difference between new data(data.0) and prev data
    let change = data.0 - component.stock;
    let increase = if change.is_positive() { "+" } else { "" };
    let color = serenity::utils::Colour(if change.is_positive() {
        0x00ff00 // Green
    } else {
        0xff0000 // Red
    });

    // Only run if stock has changed
    if change != 0 {
        if let Err(why) = ChannelId(component.channel_id as u64)
            .send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title(&component.name).url(format!(
                        "https://jlcpcb.com/parts/componentSearch?isSearch=true&searchTxt={}",
                        component.lcsc
                    ));
                    e.colour(color);
                    e.thumbnail(&data.1);
                    e.timestamp(&Utc::now());
                    e.field(
                        "Stock",
                        format!(
                            "{stock} ({increase}{value})",
                            stock = data.0,
                            value = change,
                            increase = increase
                        ),
                        false,
                    );
                    e.field("Previous Stock", component.stock, false);
                    e.field(
                        "LCSC Number",
                        format!("{}\n{}", component.lcsc.as_str(), data.3),
                        false,
                    );
                    e.field("Price", data.2, false);
                    e
                })
            })
            .await
        {
            eprintln!("Error sending message: {:?}", why);
        };
    }
    // Check if role needs to be pinged
    if component.stock == 0 && data.0 > 0 && component.role_id != 0 {
        ChannelId(component.channel_id as u64)
            .say(&ctx.http, format!("<@&{}>", component.role_id))
            .await
            .expect("Error");
        println!("Pinged role for {}", component.name);
    }

    // Create component struct out of new and old info
    Ok(Component {
        stock: data.0,
        name: component.name.clone(),
        lcsc: component.lcsc.clone(),
        enabled: component.enabled,
        channel_id: component.channel_id,
        role_id: component.role_id,
    })
}

// Main function for jlc stock check
pub async fn jlc_stock_check(ctx: Arc<Context>) {
    // Read all components from DB and read postgres pool from ctx
    let component_list = get_components(&ctx).await;
    let data_read = ctx.data.read().await;
    let pool = data_read.get::<DatabasePool>().unwrap();

    // Initialize futures vector with dummy variable so rust can get the type hint
    let mut futures = vec![print_stock_data(
        Arc::clone(&ctx),
        component_list[0].clone(),
    )];
    // Remove dummy variable
    futures.pop();

    // Add futures for all components into initialized vector
    for component in component_list {
        if component.enabled {
            futures.push(print_stock_data(Arc::clone(&ctx), component));
        }
    }
    // Execute all futures with join_all in parallel
    let results = join_all(futures).await;

    // Iterate over results and update database
    // Print debug info to console
    for result in results {
        let data = match result {
            Ok(data) => data,
            Err(_err) => continue,
        };

        println!("Sent stock for {}, Stock:{}", data.name, data.stock);
        // binding params wasn't working
        let update = format!(
            "UPDATE components SET stock = {} WHERE lcsc='{}'",
            data.stock, data.lcsc
        );
        sqlx::query(update.as_str()).execute(pool).await.unwrap();
    }
}
