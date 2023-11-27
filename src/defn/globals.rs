use std::collections::HashMap;
use std::sync::Arc;
use serenity::prelude::*;
use crate::defn::command::CommandHandler;

pub type CommandMap = Arc<RwLock<HashMap< &'static str, Arc<dyn CommandHandler> >>>;

pub struct CommandMapTmk;
impl TypeMapKey for CommandMapTmk {
    type Value = CommandMap;
}
