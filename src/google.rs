
use std::collections::{HashSet, HashMap};
use reqwest::header::USER_AGENT;
use reqwest::Client;
use select::document::Document;
use select::predicate::*;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Clone)]
pub struct Section {
    pub link: String,
    pub title: String,
}


pub fn search_from_google(query: &str, limit: u32) -> Result<Vec<Section>, String> {
    let url = format!(
        "https://www.google.com/search?q={}&gws_rd=ssl&num={}",
        query, limit
    );

    let body = Client::new()
        .get(url.as_str())
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.10; rv:34.0) Gecko/20100101 Firefox/34.0")
        .send()
        .unwrap()
        .text()
        .unwrap();

    let mut sections: Vec<Section> = Vec::new();
    let document = Document::from(body.as_str());

    for node in document.find(
        Attr("id", "ires")
            .descendant(Class("bkWMgd"))
            .descendant(Class("r"))
            .descendant(Name("a")),
    ) {
        let link = node.attr("href").unwrap();
        for new_node in node.find(Class("LC20lb")) {
            sections.push(Section { title: new_node.text(), link: link.to_string()} )
        }
    }

    Ok(sections)
}
