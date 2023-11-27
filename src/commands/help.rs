use async_trait::async_trait;
use crate::defn::command::{ Command, Context, Message, ClientData, CommandHandler };
use crate::defn::globals::CommandMapTmk;
use crate::nom_util as util;

pub struct HelpHandler;

fn right_pad(s: impl std::fmt::Display, n: usize) -> String {
    let mut s = format!("{}", s);
    let m = s.len();
    for _ in m..n { s.push(' '); }

    s
}

#[async_trait]
impl CommandHandler for HelpHandler {
    async fn whatis(&self, name: &str) -> String {
        if name == "h" {
            "Alias of `help`".into()
        } else {
            "Print all existing commands".into()
        }
    }
    async fn register(&mut self, _: ClientData) -> Vec<&'static str> {
        vec!["help", "h"]
    }
    async fn call(&self, Command { name: _, args, rest }: Command, ctx: Context, msg: Message) {
        // NOTE at this stage, `help` simply lists all the available commands

        // parse (validate) arguments
        if args.len() != 0 {
            let _: Option<_> = util::try_reply(ctx, msg, "```\nhelp: does not accept arguments (yet)\n```").await;
            return;
        }
        if rest.len() != 0 {
            let _: Option<_> = util::try_reply(ctx, msg, "```\nhelp: does not accept a rest-field\n```").await;
            return;
        }

        let mut v = vec![];
        {
            let data = ctx.data.read().await;
            let cm = data
                .get::<CommandMapTmk>().expect("Command map does not exist!")
                .read().await;
            let mut keys: Vec<&str> = cm.keys().map(|&s| s).collect();
            let max_key_len = keys.iter().map(|k| k.len()).max().unwrap_or(0);
            keys.sort();
            for k in keys.into_iter() {
                v.push(format!("{} - {}", right_pad(k, max_key_len), cm.get(k).unwrap().whatis(k).await));
            }
        }

        if v.len() == 0 {
            let _: Option<_> = util::try_reply(ctx, msg, "There is no help. (???)").await;
        } else {
            let _: Option<_> = util::try_reply(ctx, msg, format!("```\n{}\n```", v.join("\n"))).await;
        }
    }
}
