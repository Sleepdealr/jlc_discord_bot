use std::fs;
use std::sync::Arc;

use chrono::Utc;
use serde_json::Value;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use serenity::utils::{Color, Colour};

use crate::keys::{Component, Components, Data};

pub fn read_components_json(path: &str) -> Components {
    let file = fs::File::open(path).expect("file should open read only");
    serde_json::from_reader(file).expect("file should be proper JSON")
}

pub async fn get_jlc_stock(lcsc: &str) -> Result<Data, reqwest::Error> {
    let response: Value = reqwest::Client::new()
        .post("https://jlcpcb.com/shoppingCart/smtGood/selectSmtComponentList")
        .json(&serde_json::json!({ "keyword": lcsc }))
        .send()
        .await?
        .json()
        .await?;

    let jlc_stock = response["data"]["componentPageInfo"]["list"][0]["stockCount"]
        .as_i64()
        .unwrap();

    let image_url = response["data"]["componentPageInfo"]["list"][0]["componentImageUrl"]
        .as_str()
        .unwrap()
        .to_string();
    let data = Data {
        stock: jlc_stock,
        image_url,
    };
    Ok(data)
}

pub async fn print_stock_data(ctx: Arc<Context>, component: &Component) -> i64 {
    let data = get_jlc_stock(&component.lcsc)
        .await
        .expect("Error getting stock data");
    let change = data.stock - component.prev_stock;
    let increase = if change.is_positive() {
        "+"
    }else{
        ""
    };
    let color  = if change.is_positive() {
        0x00ff00
    }else{
       0xff0000
    };

    if data.stock != component.prev_stock {
        if let Err(why) = ChannelId(component.channel_id)
            .send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title(&component.name).url(format!(
                        "https://jlcpcb.com/parts/componentSearch?isSearch=true&searchTxt={}",
                        component.lcsc
                    ));
                    e.colour(color);
                    e.thumbnail(&data.image_url);
                    e.timestamp(&Utc::now());
                    e.field("Stock", format!("{stock} ({increase}{value})",stock = data.stock, value = change, increase = increase ), false);
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
