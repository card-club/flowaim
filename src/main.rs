use clap::{Parser, Subcommand};
use config::Config;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
mod api;
mod gitignore_check;
use api::{auth, schemas};
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect};
extern crate toml;

use crate::api::stats;
use crate::api::tables;

static FLOWAIM_CONFIG_FILENAME: &str = ".flowaim/config.toml";
static FLOWAIM_CONFIG_FOLDER: &str = ".flowaim";

#[derive(Serialize)]
struct CLIConfig {
    user_id: String,
    private_key: String,
    public_key: String,
    api_url: String,
}

#[derive(Parser)]
#[command(name = "FlowAim")]
#[command(author = "Charif Mews")]
#[command(version = "0.1")]
#[command(about = "Easily setup and load fake data in your SxT cryptographically verified analytics pipeline", long_about = None)]

struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Setup schema and tables for test/production environments
    Setup {},
    /// Load dummy data into tables
    Load {
        #[arg(short, long)]
        env: String,
        #[arg(short, long)]
        publisher_id: i64,
        #[arg(short, long)]
        deck_id: i64,
        #[arg(long)]
        event_type: String,
    },
    /// Show stats
    Stats {
        #[arg(short, long)]
        env: String,
    },
    /// Destroy schema
    Destroy {},
}

fn main() -> Result<(), ureq::Error> {
    gitignore_check::add_flowaim_config_filename_if_needed(FLOWAIM_CONFIG_FOLDER).unwrap();

    let cli = Cli::parse();

    if Path::new(FLOWAIM_CONFIG_FILENAME).exists() {
        println!("Config {} exists", FLOWAIM_CONFIG_FILENAME);
    } else {
        println!(
            "The config {} does not exist. Creating it now...",
            FLOWAIM_CONFIG_FILENAME
        );
        fs::create_dir_all(FLOWAIM_CONFIG_FOLDER)?;
        let cli_config = CLIConfig {
            user_id: "".to_string(),
            private_key: "".to_string(),
            public_key: "".to_string(),
            api_url: "".to_string(),
        };

        let toml_str = toml::to_string(&cli_config).unwrap();
        let mut file = File::create(FLOWAIM_CONFIG_FILENAME)?;
        file.write_all(toml_str.as_bytes())?;
        println!(
            "Config {} created, fill in the values before running the CLI again",
            FLOWAIM_CONFIG_FILENAME
        );
        return Ok(());
    }

    let settings = Config::builder()
        .add_source(config::File::with_name(FLOWAIM_CONFIG_FILENAME))
        .add_source(config::Environment::with_prefix("FLOWAIM"))
        .build()
        .unwrap();

    let configsettings = settings
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();
    let acces_token = auth::get_acces_token(&configsettings).unwrap();

    match &cli.command {
        Commands::Setup {} => {
            println!("Start SxT setup");
            let org_name: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("What is your organization/project name? (use enter to confirm)")
                .interact_text()
                .unwrap();

            let env_choices = &["dev", "test", "stag", "prod"];
            let env_defaults = &[true, false, false, false];
            let envs = MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Which environments do you want to setup? (use spacebar to select and enter to confirm)")
                .items(&env_choices[..])
                .defaults(&env_defaults[..])
                .interact()
                .unwrap();

            if envs.is_empty() {
                println!("You did not select an environment :( Setup cancelled");
                return Ok(());
            } else {
                println!("Creating schema: {}", org_name.clone());
                schemas::create_schema(
                    &org_name,
                    configsettings.get("api_url").unwrap(),
                    &acces_token,
                )
                .unwrap();

                for env in envs {
                    println!("Creating table: {}.{}", org_name.clone(), env_choices[env]);
                    tables::create_table(
                        &org_name,
                        env_choices[env],
                        configsettings.get("api_url").unwrap(),
                        &acces_token,
                    )
                    .unwrap();
                }
                println!("Environment is ready!");
            }
        }
        Commands::Load {
            env,
            publisher_id,
            deck_id,
            event_type,
        } => {
            let table_settings = Config::builder()
                .add_source(config::File::with_name(&format!(".flowaim/{}.toml", env)))
                .build()
                .unwrap();

            let table_configsettings = table_settings
                .try_deserialize::<HashMap<String, String>>()
                .unwrap();

            let table_rows: i8 = Input::<i8>::with_theme(&ColorfulTheme::default())
                .with_prompt(
                    "How many rows do you want to load into your table? (use enter to confirm)",
                )
                .interact_text()
                .unwrap();

            println!(
                "Loading {} rows into {}. One moment please",
                table_rows, table_configsettings["resource_name"]
            );

            tables::insert_rows(
                &table_configsettings["resource_name"],
                &configsettings.get("api_url").unwrap(),
                &acces_token,
                &table_configsettings["biscuit"],
                &table_rows,
                publisher_id,
                deck_id,
                event_type,
            )
            .unwrap();
            println!(
                "Finished loading {} rows into {}.",
                table_rows, table_configsettings["resource_name"]
            );
        }
        Commands::Stats { env } => {
            let table_settings = Config::builder()
                .add_source(config::File::with_name(&format!(".flowaim/{}.toml", env)))
                .build()
                .unwrap();

            let table_configsettings = table_settings
                .try_deserialize::<HashMap<String, String>>()
                .unwrap();

            stats::print_stats(
                &table_configsettings["resource_name"],
                &configsettings.get("api_url").unwrap(),
                &acces_token,
                &table_configsettings["biscuit"],
            )
            .unwrap();
        }
        Commands::Destroy {} => {
            let org_name: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(
                    "Which organisaton/project do you want to delete? (use enter to confirm)",
                )
                .interact_text()
                .unwrap();

            schemas::drop_schema(
                &org_name,
                configsettings.get("api_url").unwrap(),
                &acces_token,
            )
            .unwrap();
            println!("Finished deleting schema {}.", org_name);
        }
    }
    Ok(())
}
