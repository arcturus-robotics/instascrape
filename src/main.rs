use scraper::{Html, Selector};
use std::{fs::File, io::Read};

fn main() {
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

    let followers = content
        .split_terminator(' ')
        .next()
        .unwrap()
        .parse::<u64>()
        .unwrap();

    println!("{}", followers);
}
