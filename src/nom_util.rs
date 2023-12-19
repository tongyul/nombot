use crate::defn::command::{ Context, Message };

pub async fn try_reply(ctx: &Context, msg: &Message, reply: impl std::fmt::Display) -> Option<Message> {
    match msg.channel_id.say(&ctx.http, reply).await {
        Ok(m) => Some(m),
        Err(why) => {
            println!("Error sending message: {why:?}");
            None
        }
    }
}
