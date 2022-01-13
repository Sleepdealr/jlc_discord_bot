use std::fs;
use std::fs::File;
use std::sync::Arc;

use crate::keys::{Component, Components, Data, Datasheets};
use chrono::Utc;
use serde_json::Value;
use serenity::model::id::ChannelId;
use serenity::prelude::*;

// Yeah I can probably use an enum for this
pub fn read_components_json(path: &str) -> Components {
    let file = fs::File::open(path).expect("file should open read only");
    serde_json::from_reader(file).expect("file should be proper JSON")
}

pub fn read_datasheet_json(path: &str) -> Datasheets {
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

    // Error handling on image isn't necessary since it should have one if it has stock
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

pub async fn print_stock_data(
    ctx: Arc<Context>,
    component: &Component,
) -> Result<i64, reqwest::Error> {
    let request = get_jlc_stock(&component.lcsc).await;
    let data = match request {
        Ok(res) => res,
        Err(e) => return Err(e),
    };
    let change = data.stock - component.prev_stock;
    let increase = if change.is_positive() { "+" } else { "" };
    let color = if change.is_positive() {
        0x00ff00
    } else {
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
                    e.field(
                        "Stock",
                        format!(
                            "{stock} ({increase}{value})",
                            stock = data.stock,
                            value = change,
                            increase = increase
                        ),
                        false,
                    );
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
    Ok(data.stock)
}

pub async fn jlc_stock_check(ctx: Arc<Context>) {
    let mut component_list: Components = read_components_json("config/components.json");
    for component in &mut component_list.components {
        if component.enabled {
            let response = print_stock_data(Arc::clone(&ctx), component).await;
            let data = match response {
                Ok(data) => data,
                Err(_err) => continue,
            };
            println!("Sent stock for {}, Stock:{}", component.name, data);
            component.prev_stock = data;
        }
    }
    serde_json::to_writer_pretty(
        &File::create("config/components.json").expect("File creation error"),
        &component_list,
    )
    .expect("Error writing file");
}
