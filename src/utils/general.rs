use chrono::Utc;
use serenity::prelude::Context;

use crate::keys::Uptime;

pub async fn get_uptime(ctx: &Context) -> String {
    let uptime = {
        let data = ctx.data.read().await;
        match data.get::<Uptime>() {
            Some(time) => {
                if let Some(boot_time) = time.get("boot") {
                    let now = Utc::now();
                    let mut f = timeago::Formatter::new();
                    f.num_items(4);
                    f.ago("");

                    f.convert_chrono(boot_time.to_owned(), now)
                } else {
                    "Uptime not available".to_owned()
                }
            }
            None => "Uptime not available.".to_owned(),
        }
    };
    uptime
}
