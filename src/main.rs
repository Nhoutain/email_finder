extern crate clap;
extern crate clap_log_flag;
extern crate clap_verbosity_flag;
extern crate chrono;
extern crate csv;
extern crate ctrlc;
extern crate dialoguer;
extern crate dirs;
extern crate indicatif;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate select;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate structopt;
extern crate tempfile;
extern crate reqwest;
extern crate url;

mod find;
mod google;
mod write;

use std::collections::{HashSet, HashMap};
use std::fs::OpenOptions;
use std::path::PathBuf;
use structopt::StructOpt;


lazy_static! {
    pub static ref HOME_DIR: PathBuf = dirs::home_dir().expect("HOME variable is not defined. Are you on a linux OS?");

    pub static ref LOG_DIR_FILE: String = format!("{}/.emailFinder", HOME_DIR.display());
    pub static ref LOG_FILE: String = format!("{}/.emailFinder/emailFinder.log", HOME_DIR.display());
}

#[derive(Debug, Serialize)]
struct Record {
    link: String,
    section: String,
    mail: String
}

#[derive(StructOpt, Debug)]
#[structopt(name = "email_finder", about = "Search email adress")]
pub enum Finder {

    #[structopt(name = "google", about = "Search from google")]
    Google {
        /// Search 
        #[structopt(name = "search", long, short)]
        search: String,

        #[structopt(name = "limit", long, short)]
        limit: u32,

        #[structopt(name = "depth", long, short)]
        depth: u32,

        #[structopt(flatten)]
        output: Output,

        #[structopt(flatten)]
        verbose: clap_verbosity_flag::Verbosity,

        #[structopt(flatten)]
        log: clap_log_flag::Log,

    }
}

#[derive(StructOpt, Debug)]
pub struct Output {

    #[structopt(name="output", long, short, parse(from_os_str))]
    file: Option<PathBuf>
}

fn main() -> Result<(), String> {


    let mut result: HashMap<String, (google::Section, HashSet<String>)> = HashMap::new();

    match Finder::from_args() {
        Finder::Google { search, limit, depth, output, verbose, log } => {

            log.log_all(Some(verbose.log_level()));

            for section in google::search_from_google(&search, limit)? {
                result.insert(section.link.clone(), (section.clone(), find::find(&section.link, depth)));
            }

            write(result, output);
        }
    }

    Ok(())
}

fn write(result: HashMap<String, (google::Section, HashSet<String>)>, output: Output) {

    if output.file.is_some() {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.file.unwrap()).unwrap();

        let mut wtr = csv::Writer::from_writer(file);

        for (link, (section, emails)) in result {
            for email in emails {
                wtr.serialize(Record {
                    link: link.clone() ,
                    section: section.title.clone(),
                    mail: email
                });
            }
        }

        wtr.flush();
    }
}
