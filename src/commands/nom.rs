use async_trait::async_trait;
use crate::defn::command::{
    Arg, Command, Context, Message, ClientData,
    CommandHandler,
};
use crate::nom_util as util;

pub struct NomHandler;
#[async_trait]
impl CommandHandler for NomHandler {
    async fn whatis(&self, _: &str) -> String { "Nommers. ('!' for more enthusiasm, '.' for less)".into() }
    async fn register(&mut self, _: ClientData) -> Vec<&'static str> { vec!["nom"] }
    async fn call(&self, Command { name: _, args, rest }: Command, ctx: Context, msg: Message) {
        // a simple secondary parser
        if rest.len() != 0 {
            let _: Option<_> = util::try_reply(ctx, msg, "```\nnom: does not accept a rest-field\n```").await;
            return;
        }
        let mut ups = 0u32;
        let mut downs = 0u32;
        async fn report_conflict(c: Context, m: Message) {
            let _: Option<_> = util::try_reply(c, m, "```\nnom: cannot have both `!`s and `.`s\n```").await;
        }
        for a in args.iter() {
            match a {
                Arg::Pos(s) => {
                    for c in s.chars() {
                        match c {
                            '!' if downs != 0 => { let () = report_conflict(ctx, msg).await; return; }
                            '!' => { ups = (ups + 1).min(3); }
                            '.' if ups != 0 => { let () = report_conflict(ctx, msg).await; return; }
                            '.' => { downs = (downs + 1).min(2); }
                            _ => { let _: Option<_> = util::try_reply(ctx, msg, format!("```\nnom: unrecognized character {c:?}\n```")).await; return; }
                        }
                    }
                }
                Arg::Kw(..) => { let _: Option<_> = util::try_reply(ctx, msg, format!("```\nnom: does not accept keyword arguments\n```")).await; return; }
            }
        }

        // react
        let enthusiasm = 2 - downs + ups;
        let reply = match enthusiasm {
            0 => ".",
            1 => "nom.",
            2 => "nommers",
            3 => "nommers!",
            4 => "NOMMERS!!!",
            5 => "nom-mers~\nnom-mers~\nnom-nom-nom-mers~\nnom-mers~\nnom-mers~\nnom-nom-nom-mers~",
            _ => unreachable!(),
        };
        let _: Option<_> = util::try_reply(ctx, msg, reply).await;
    }
}
