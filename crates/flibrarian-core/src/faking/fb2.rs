use fake::Fake;
use fake::faker::lorem::en::Paragraphs;
use rand::Rng;

use super::names::{
    FIRST_NAMES_EN, FIRST_NAMES_RU, GENRES, LAST_NAMES_EN, LAST_NAMES_RU, MIDDLE_NAMES_RU,
    SEQUENCE_NAMES, TITLE_ADJECTIVES, TITLE_NOUNS,
};

fn pick<'a>(rng: &mut impl Rng, items: &'a [&str]) -> &'a str {
    items[rng.gen_range(0..items.len())]
}

fn generate_author_xml(rng: &mut impl Rng, lang: &str) -> String {
    let (first, last, middle) = match lang {
        "ru" => (
            pick(rng, FIRST_NAMES_RU),
            pick(rng, LAST_NAMES_RU),
            Some(pick(rng, MIDDLE_NAMES_RU)),
        ),
        _ => (pick(rng, FIRST_NAMES_EN), pick(rng, LAST_NAMES_EN), None),
    };

    let middle_tag = middle.map_or_else(String::new, |m| format!("<middle-name>{m}</middle-name>"));

    format!(
        "<author><first-name>{first}</first-name>{middle_tag}<last-name>{last}</last-name></author>"
    )
}

fn generate_title(rng: &mut impl Rng) -> String {
    let adj = pick(rng, TITLE_ADJECTIVES);
    let noun = pick(rng, TITLE_NOUNS);
    format!("The {adj} {noun}")
}

fn generate_sequence_xml(rng: &mut impl Rng) -> String {
    if rng.gen_range(0u8..3) == 0 {
        return String::new();
    }
    let name = pick(rng, SEQUENCE_NAMES);
    let number = rng.gen_range(1u32..20);
    format!(r#"<sequence name="{name}" number="{number}"/>"#)
}

fn generate_body_xml(rng: &mut impl Rng) -> String {
    let paragraphs: Vec<String> = Paragraphs(3..8).fake_with_rng(rng);
    let body = paragraphs
        .into_iter()
        .map(|p| format!("<p>{p}</p>"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("<body><section>\n{body}\n</section></body>")
}

pub fn generate_fb2_xml(rng: &mut impl Rng, lang: &str) -> String {
    let author_count = rng.gen_range(1u8..=3);
    let authors: String = (0..author_count)
        .map(|_| generate_author_xml(rng, lang))
        .collect::<Vec<_>>()
        .join("\n");

    let genre = pick(rng, GENRES);
    let title = generate_title(rng);
    let year = rng.gen_range(1950u16..2025);
    let sequence = generate_sequence_xml(rng);
    let body = generate_body_xml(rng);

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description>
<title-info>
<genre>{genre}</genre>
{authors}
<book-title>{title}</book-title>
<date>{year}</date>
{sequence}
<lang>{lang}</lang>
</title-info>
</description>
{body}
</FictionBook>"#
    )
}
