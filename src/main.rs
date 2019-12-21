use scraper::{Html, Selector};
use serenity::{
    client::Client,
    framework::standard::{
        macros::{command, group},
        CommandResult, StandardFramework,
    },
    model::channel::Message,
    prelude::*,
};
use std::{env, fs::File, io::Read};

fn get_followers() -> u64 {
    let user = {
        let mut file = File::open("./user.txt").unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        buf
    };

    let content = reqwest::get(format!("https://www.instagram.com/{}", user).as_str())
        .unwrap()
        .text()
        .unwrap();
    let document = Html::parse_document(&content);

    let selector = Selector::parse(r#"meta[property="og:description"]"#).unwrap();
    let meta = document.select(&selector).next().unwrap();
    let content = meta.value().attr("content").unwrap();

    content
        .split_terminator(' ')
        .next()
        .unwrap()
        .parse::<u64>()
        .unwrap()
}

group!({
    name: "general",
    options: {},
    commands: [followers],
});

struct Handler;

impl EventHandler for Handler {}

fn main() {
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("~"))
            .group(&GENERAL_GROUP),
    );

    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
fn followers(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, format!("{}", get_followers()))?;

    Ok(())
}
