use std::sync::Arc;

use chrono::Utc;
use serde_json::Value;
use serenity::model::id::ChannelId;
use serenity::prelude::*;

use crate::{Component, Data};

pub async fn get_jlc_stock(lcsc: &str) -> Result<Data, reqwest::Error> {
    let echo_json: Value = reqwest::Client::new()
        .post("https://jlcpcb.com/shoppingCart/smtGood/selectSmtComponentList")
        .json(&serde_json::json!({ "keyword": lcsc }))
        .send()
        .await?
        .json()
        .await?;

    let jlc_stock = echo_json["data"]["componentPageInfo"]["list"][0]["stockCount"]
        .as_u64()
        .unwrap();

    let image_url = echo_json["data"]["componentPageInfo"]["list"][0]["componentImageUrl"]
        .as_str()
        .unwrap()
        .to_string();
    let data = Data {
        stock: jlc_stock,
        image_url,
    };
    Ok(data)
}

pub async fn print_stock_data(ctx: Arc<Context>, component: &Component) -> u64 {
    let data = get_jlc_stock(&component.lcsc)
        .await
        .expect("Error getting stock data");
    if data.stock != component.prev_stock {
        if let Err(why) = ChannelId(component.channel_id)
            .send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title(&component.name).url(format!(
                        "https://jlcpcb.com/parts/componentSearch?isSearch=true&searchTxt={}",
                        component.lcsc
                    ));
                    e.thumbnail(&data.image_url);
                    e.timestamp(&Utc::now());
                    e.field("Stock", format!("{}", data.stock), false);
                    e.field("Previous Stock", component.prev_stock, false);
                    e.field("LCSC Number", component.lcsc.as_str(), false);
                    e
                })
            })
            .await
        {
            eprintln!("Error sending message: {:?}", why);
        };
    }
    if component.prev_stock == 0 && data.stock > 0 && component.role_id != 0 {
        ChannelId(component.channel_id)
            .say(&ctx.http, format!("<@&{}>", component.role_id))
            .await
            .expect("Error");
        println!("Pinged role for {}", component.name);
    }
    data.stock
}