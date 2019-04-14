
use std::collections::{HashSet, VecDeque};
use reqwest::header::USER_AGENT;
use reqwest::Client;
use regex::Regex;
use url::Url;

lazy_static! {
    static ref MAIL_RE: Regex = Regex::new(
	r"([a-zA-Z0-9._-]+@[a-zA-Z0-9._-]+\.[a-zA-Z0-9_-]+)"
    ).expect("Unable to create regex for mail");

    static ref LINK_RE: Regex = Regex::new(
	"(?:(?:https?|ftp):\\/\\/|\\b(?:[a-z\\d]+\\.))(?:(?:[^\\s()<>]+|\\((?:[^\\s()<>]+|(?:\\([^\\s()<>]+\\)))?\\))+(?:\\((?:[^\\s()<>]+|(?:\\(?:[^\\s()<>]+\\)))?\\)|[^\\s`!()\\[\\]{};:'\".,<>?«»“”‘’]))?"
    ).expect("Unable to create regex for link");
}


pub fn find(link: &String, mut limit: u32) -> HashSet<String> {
    info!("Search email adresses for link {}", link);
    let link_url = match Url::parse(link) {
        Ok(url) => url,
        Err(e) => {
            error!("Unable to parse link to url: {}", link);
            return HashSet::new();
        }
    };

    let mut mails: HashSet<String> = HashSet::new();

    // Initialize search
    debug!("Initialize search for {} with limit {}", link, limit);
    let mut explored: HashSet<String> = HashSet::new();
    let mut to_explore: VecDeque<String> = VecDeque::new();
    to_explore.push_back(link.clone());


    // Start search
    debug!("Start search for {}", link);
    let mut current_link: String = link.to_string().clone();

    while limit > 0 && ! to_explore.is_empty() {
        match find_mails_links(&current_link) {
            (find_mails, find_links) => {
                for find_mail in find_mails {
                    mails.insert(find_mail);
                }

                for find_link in find_links {
                    if ! explored.contains(&find_link) && in_domain(&link_url, &find_link) {
                        to_explore.push_back(find_link);
                    }
                }
            }
        };

        explored.insert(current_link.clone());
        limit = limit - 1;

        current_link = to_explore.pop_front().expect("To explore is empty");
    }

    mails

}

fn in_domain(base: &Url, link: &String) -> bool {
    match Url::parse(link) {
        Ok(url) => {
            debug!("Parse domain link {} to {:?}", link, url);
            base.domain() == url.domain()
        },
        Err(e) => {
            debug!("Not in same domain link {} for base {} for error {:?}", link, base, e);
            false
        }
    }
}

pub fn find_mails_links(link: &String) -> (HashSet<String>, HashSet<String>) {
    debug!("Search email adresses for link {}", link);

    let body = match Client::new()
        .get(link)
        .header(USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.10; rv:34.0) Gecko/20100101 Firefox/34.0")
        .send() {
            Ok(mut result) => match result.text() {
                Ok(text) => text,
                Err(e) => {
                    debug!("Unable to contact link: {} due to {}", link, e);
                    String::new()
                }
            },

            Err(e) =>  {
                debug!("Unable to contact link: {} due to {}", link, e);
                String::new()
            }
        };

    let mut mails: HashSet<String> = HashSet::new();
    for mail_cap in MAIL_RE.captures_iter(&body) {
        let mail = mail_cap.get(1).unwrap().as_str().to_string();
        debug!("Find mail: {}", mail);
        mails.insert(mail);
    }

    let mut links: HashSet<String> = HashSet::new();
    for link_cap in MAIL_RE.captures_iter(&body) {
        let link = link_cap.get(1).unwrap().as_str().to_string();
        debug!("Find link: {}", link);
        links.insert(link);
    }

    (mails, links)
}
