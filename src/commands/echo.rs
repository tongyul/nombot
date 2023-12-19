use async_trait::async_trait;
use crate::defn::command::{
    Arg, Command, Context, Message, ClientData,
    CommandHandler,
};
use crate::nom_util as util;

const HELP_STR: &'static str =
"## Examples
Let nombot say hello world
```
nom/echo/Hello, world!
```
Let nombot say aehmttw
```
nom/echo sorted=1/matthew
```
Let nombot say world hello on two lines
```
nom/echo reversed=1 scope=line/Hello,
world!
```
Short hands are supported, and empty lines are ignored when using line scope
```
nom/echo -srl/
Bananas
Apples
Cherries
```
## Usage
Print help (this message)
```
nom/echo help
```
Echo-back the rest-field, transformed according to options
```
nom/echo [-sSrRlc] sorted=[0|1] reversed=[0|1] scope=[char|line]
```";

pub struct EchoHandler;

#[async_trait]
impl CommandHandler for EchoHandler {
    async fn whatis(&self, _: &str) -> String { "Echo-back the rest-field".into() }
    async fn register(&mut self, _: ClientData) -> Vec<&'static str> {
        vec!["echo"]
    }
    async fn call(&self, Command { name: _, args, rest }: Command, ctx: Context, msg: Message) {
        // parse the command a second time for subcommands, flags, and options
        enum Scope {
            Lines,
            Characters,
        }
        enum Sc { // subcommand
            Echo { sorted: bool, reversed: bool, scope: Scope },
            Help,
            Error { why: String },
        }
        let mut sorted_opt = None;
        let mut reversed_opt = None;
        let mut scope_opt = None;
        let mut itr = args.iter();
        let sc = 'Z: loop {
            match itr.next() {
                None =>
                    break Sc::Echo {
                        sorted: sorted_opt.unwrap_or(false),
                        reversed: reversed_opt.unwrap_or(false),
                        scope: scope_opt.unwrap_or(Scope::Characters),
                    },
                Some(Arg::Pos(s)) if s.chars().count() >= 1 && s.chars().nth(0).unwrap() == '-' => {
                    let mut itr = s.chars();
                    let _ = itr.next();
                    for c in itr {
                        match c {
                            's' | 'S' if sorted_opt.is_some() =>
                                break 'Z Sc::Error { why: "the 'sorted' option is set multiple times".into() },
                            's' | 'S' => { sorted_opt = Some(c == 's'); }
                            'r' | 'R' if reversed_opt.is_some() =>
                                break 'Z Sc::Error { why: "the 'reversed' option is set multiple times".into() },
                            'r' | 'R' => { reversed_opt = Some(c == 'r'); }
                            'l' | 'c' if scope_opt.is_some() =>
                                break 'Z Sc::Error { why: "the 'scope' option is set multiple times".into() },
                            'l' | 'c' => {
                                scope_opt = Some(if c == 'c' { Scope::Characters } else { Scope::Lines });
                            }
                            _ =>
                                break 'Z Sc::Error {
                                    why: format!("unrecognized flag shorthand {c:?}; available shorthands: sSrRlc"),
                                },
                        }
                    }
                }
                Some(Arg::Pos(s)) if s == "help" => break Sc::Help,
                Some(Arg::Pos(_)) =>
                    break Sc::Error {
                        why: format!("does not accept non-flag positional arguments; use the rest-field instead."),
                    },
                Some(Arg::Kw(k, _)) if k == "sorted" && sorted_opt.is_some() =>
                    break Sc::Error { why: "the 'sorted' option is set multiple times".into() },
                Some(Arg::Kw(k, v)) if k == "sorted" => {
                    let s = v == "1";
                    if s || v == "0" {
                        sorted_opt = Some(s);
                    } else {
                        break 'Z Sc::Error { why: "the 'sorted' option is boolean (0 or 1)".into() };
                    }
                }
                Some(Arg::Kw(k, _)) if k == "reversed" && reversed_opt.is_some() =>
                    break Sc::Error { why: "the 'reversed' option is set multiple times".into() },
                Some(Arg::Kw(k, v)) if k == "reversed" => {
                    let r = v == "1";
                    if r || v == "0" {
                        reversed_opt = Some(r);
                    } else {
                        break 'Z Sc::Error { why: "the 'reversed' option is boolean (0 or 1)".into() };
                    }
                }
                Some(Arg::Kw(k, _)) if k == "scope" && scope_opt.is_some() => {
                    break Sc::Error { why: "the 'scope' option is set multiple times".into() };
                }
                Some(Arg::Kw(k, v)) if k == "scope" => {
                    let c = v == "char";
                    if c || v == "line" {
                        scope_opt = Some(if c { Scope::Characters } else { Scope::Lines });
                    } else {
                        break 'Z Sc::Error { why: "the 'scope' option has value 'char' or 'line'".into() };
                    }
                }
                Some(Arg::Kw(k, _)) =>
                    break Sc::Error {
                        why: format!("unrecognized option {k:?}; available options: sorted, reversed"),
                    },
            }
        };

        // act on the subcommands
        async fn make_echo_reply(ctx: Context, msg: Message, reply: String) {
            let _: Option<_> = if msg.author.id == ctx.cache.current_user().id
                && reply.len() >= "nom/echo".len() && &reply[.."nom/echo".len()] == "nom/echo"
            {
                util::try_reply(&ctx, &msg, "nombot refuses to `nom/echo`-bomb this channel. :/").await
            } else if reply.len() == 0 {
                util::try_reply(&ctx, &msg, "nombot cannot send an empty message. :/").await
            } else {
                util::try_reply(&ctx, &msg, reply).await
            };
        }

        match sc {
            Sc::Error { why } =>
                { let _: Option<_> = util::try_reply(&ctx, &msg, format!("```echo: {why}```")).await; }
            Sc::Help =>
                { let _: Option<_> = util::try_reply(&ctx, &msg, HELP_STR).await; }
            Sc::Echo { sorted, reversed, scope: Scope::Characters } => {
                let reply = match (sorted, reversed) {
                    (false, false) => rest,
                    (false, true) => rest.chars().rev().collect(),
                    (true, false) => {
                        let mut v: Vec<_> = rest.chars().collect();
                        v.sort();
                        v.into_iter().collect()
                    }
                    (true, true) => {
                        let mut v: Vec<_> = rest.chars().collect();
                        v.sort();
                        v.into_iter().rev().collect()
                    }
                };
                make_echo_reply(ctx, msg, reply).await;
            }
            Sc::Echo { sorted, reversed, scope: Scope::Lines } => {
                let mut lines: Vec<&str> = rest.split('\n').filter(|l| l.len() >= 1).collect();
                if sorted { lines.sort(); }
                if reversed { lines.reverse(); }
                let reply = lines.join("\n");
                make_echo_reply(ctx, msg, reply).await;
            }
        };
    }
}
