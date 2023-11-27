use std::sync::Arc;
use async_trait::async_trait;
use serenity::prelude::*;
pub use serenity::prelude::Context;
pub use serenity::model::channel::Message;
pub use crate::nom_args::{ Arg, Command };

pub type ClientData = Arc<RwLock<TypeMap>>;

#[async_trait]
pub trait CommandHandler: Send + Sync {
    async fn register(&mut self, data: ClientData) -> Vec<&'static str>;
    async fn call(&self, cmd: Command, ctx: Context, msg: Message);
    async fn whatis(&self, _: &str) -> String { "(nothing appropriate)".into() }
}
