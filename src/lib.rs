use clap::{Arg, ArgAction, Command};
use serde::{Deserialize, Serialize};
use skim::prelude::{Skim, SkimItemReader, SkimItemReaderOption, SkimOptionsBuilder};
use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::path::Path;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Args {
    open: bool,
    add: Option<Vec<String>>,
    del: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Website {
    name: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    websites: Vec<Website>,
}

impl Config {
    fn load() -> Self {
        let home_dir = dirs::home_dir().expect("Could not find config directory");
        let config_file_path = home_dir.join(".config/fzweb/config.json");

        if Path::new(&config_file_path).exists() {
            let content = fs::read_to_string(config_file_path).expect("Failed to read config file");
            serde_json::from_str(&content).unwrap_or_else(|_| Config { websites: vec![] })
        } else {
            Config { websites: vec![] }
        }
    }

    fn save(&self) {
        let home_dir = dirs::home_dir().expect("Could not find config directory");
        let config_file_path = home_dir.join(".config/fzweb/config.json");
        let config_dir_path = home_dir.join(".config/fzweb");

        if !config_dir_path.exists() {
            fs::create_dir_all(config_dir_path).expect("Failed to create config directory");
        }

        let content = serde_json::to_string_pretty(self).expect("Failed to serialize config");
        fs::write(config_file_path, content).expect("Failed to write config file");
    }

    fn add_website(&mut self, name: String, url: String) {
        if self.websites.iter().any(|w| w.name == name) {
            println!("Error: '{}' already exists.", name);
            return;
        }
        self.websites.push(Website { name, url });
        self.save();
        println!("Added successfully!");
    }

    fn remove_website(&mut self, name: String) {
        let original_len = self.websites.len();
        self.websites.retain(|w| w.name != name);
        if self.websites.len() < original_len {
            self.save();
            println!("Deleted '{}'.", name);
        } else {
            println!("Error: '{}' not found.", name);
        }
    }

    fn open_website(&self) {
        let names = self
            .websites
            .iter()
            .map(|website| website.name.clone())
            .collect();

        if let Ok(name) = select(names) {
            println!("{} is selected", name);
            if let Some(website) = self.websites.iter().find(|w| w.name == name) {
                if let Err(e) = open::that(&website.url) {
                    eprintln!("Failed to open URL: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn cli() -> Command {
    Command::new("fzweb")
        .about("A CLI tool to manage and open websites interactively.")
        .arg(
            Arg::new("add")
                .long("add")
                .short('a')
                .value_names(vec!["name", "url"])
                .num_args(2)
                .action(ArgAction::Append)
                .help("Add a website with a name and URL"),
        )
        .arg(
            Arg::new("open")
                .long("open")
                .short('o')
                .action(ArgAction::SetTrue)
                .help("Open a website in your default browser"),
        )
        .arg(
            Arg::new("del")
                .long("del")
                .short('d')
                .action(ArgAction::Set)
                .num_args(1)
                .value_name("name")
                .help("Delete a website by name"),
        )
}

pub fn get_args() -> MyResult<Args> {
    let matches = cli().get_matches();

    Ok(Args {
        add: matches
            .get_many::<String>("add")
            .map(|s| s.map(ToString::to_string).collect()),
        del: matches.get_one::<String>("del").cloned(),
        open: matches.get_flag("open"),
    })
}

fn select(names: Vec<String>) -> MyResult<String> {
    let options = SkimOptionsBuilder::default()
        .height(String::from("100%"))
        .no_multi(true)
        .no_mouse(true)
        .build()
        .unwrap();

    let input = names.join("\n");

    let item_reader = SkimItemReader::new(SkimItemReaderOption::default());
    let items = item_reader.of_bufread(Cursor::new(input));
    let output = Skim::run_with(&options, Some(items)).ok_or("Selection aborted")?;

    if output.is_abort {
        return Err("Selection aborted".into());
    }
    output
        .selected_items
        .first()
        .map(|s| s.output().to_string())
        .ok_or_else(|| "No selection made".into())
}

pub fn run(args: Args) -> MyResult<()> {
    let mut config = Config::load();

    // init
    if config.websites.is_empty() {
        config.save();
    }

    // add
    if let Some(add_site_info) = args.add {
        if let (Some(name), Some(url)) = (add_site_info.get(0), add_site_info.get(1)) {
            config.add_website(name.clone(), url.clone());
        }
    }

    // del
    if let Some(delete_site_info) = args.del {
        config.remove_website(delete_site_info);
    }

    // open
    if args.open {
        config.open_website();
    }

    Ok(())
}
