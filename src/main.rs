use clap::{Parser, Subcommand};
use config::Config;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
mod gitignore_check;
mod api;
use api::{auth, schemas};
use dialoguer::{theme::ColorfulTheme, MultiSelect, Input};

use crate::api::tables;

static FLOWAIM_CONFIG_FILENAME: &str = ".flowaim.toml";


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
    /// Setup schema and tables for test/production environemnts
    Setup {},
    /// Load data into tables
    Load {
        #[arg(short, long)]
        schema: String,
        #[arg(short, long)]
        env: String,
        #[arg(short, long)]
        biscuit: String,
        #[arg(short, long)]
        publisher_id: i64,
        #[arg(short, long)]
        deck_id: i64,
        #[arg(long)]
        event_type: String,

    },
    /// Load data into tables
    Destroy {}
}

fn main() -> Result<(), ureq::Error> {
    gitignore_check::add_flowaim_config_filename_if_needed(FLOWAIM_CONFIG_FILENAME).unwrap();

    let cli = Cli::parse();

    if Path::new(FLOWAIM_CONFIG_FILENAME).exists() {
        println!("Config {} exists", FLOWAIM_CONFIG_FILENAME);
    } else {
        println!("The config {} does not exist. Creating it now...", FLOWAIM_CONFIG_FILENAME);

        let mut configfile = File::create(FLOWAIM_CONFIG_FILENAME)?;

        configfile.write_all(b"test=\"this\"\ntwo=2")?;
    }

    let settings = Config::builder()
        .add_source(config::File::with_name(FLOWAIM_CONFIG_FILENAME))
        .add_source(config::Environment::with_prefix("FLOWAIM"))
        .build()
        .unwrap();

    let configsettings = settings
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    match &cli.command {
        Commands::Setup {} => {
            let acces_token = auth::get_acces_token(&configsettings).unwrap();
            println!("Start SxT setup");
            let org_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("What is your organization/project name? (use enter to confirm)")
            .interact_text()
            .unwrap();

            let env_choices = &[
                "dev",
                "test",
                "stag",
                "prod",
            ];
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
                schemas::create_schema(&org_name, configsettings.get("SXT_API_URL").unwrap() , &acces_token).unwrap();

                for env in envs {
                    println!("Creating table: {}.{}", org_name.clone(), env_choices[env]);
                    tables::create_table(&org_name, env_choices[env], configsettings.get("SXT_API_URL").unwrap() , &acces_token).unwrap();
                    println!("  {}", env_choices[env]);
                }
            }

        }
        Commands::Load { schema, env, biscuit, publisher_id, deck_id, event_type} => {
            let table_rows: i8 = Input::<i8>::with_theme(&ColorfulTheme::default())
            .with_prompt("How many rows do you want to load into your table? (use enter to confirm)")
            .interact_text()
            .unwrap();

            println!("Loading {} rows into {}.{}. One moment please", table_rows, schema, env);
            let acces_token = auth::get_acces_token(&configsettings).unwrap();
            tables::insert_rows(schema, env, &configsettings.get("SXT_API_URL").unwrap(), &acces_token, biscuit, &table_rows, publisher_id, deck_id, event_type).unwrap();
            println!("Finished loading {} rows into {}.{}.", table_rows, schema, env);

        }
        Commands::Destroy {} => {
            let acces_token = auth::get_acces_token(&configsettings).unwrap();
            let org_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Which organisaton/project do you want to delete? (use enter to confirm)")
            .interact_text()
            .unwrap();
            
            schemas::drop_schema(&org_name, configsettings.get("SXT_API_URL").unwrap() , &acces_token).unwrap();
            println!("Finished deleting schema {}.", org_name);
        }
    }
    Ok(())
}
