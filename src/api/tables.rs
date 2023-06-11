extern crate chrono;
extern crate rand;

use biscuit_auth::{error, macros::*, KeyPair};
use chrono::{DateTime, NaiveDateTime, Utc};
use rand::Rng;
use serde::Serialize;
use std::{fs::File, io::Write};

#[derive(Serialize)]
struct TableConfig {
    resource_name: String,
    table_private_key: String,
    table_public_key: String,
    biscuit: String,
}

fn create_biscuit(resource_name: String, root: KeyPair) -> Result<String, error::Token> {
    let schema_table_name = resource_name.to_lowercase();
    let biscuit = biscuit!(
        r#"
        sxt:capability("ddl_create", {schema_table_name});
        sxt:capability("ddl_alter", {schema_table_name});
        sxt:capability("ddl_drop", {schema_table_name});
        sxt:capability("dml_insert", {schema_table_name});
        sxt:capability("dql_select", {schema_table_name});
        sxt:capability("kafka_icm_create", {schema_table_name});
        sxt:capability("kafka_icm_read", {schema_table_name});
        sxt:capability("kafka_icm_update", {schema_table_name});
    "#
    )
    .build(&root)?;

    Ok(biscuit.to_base64()?.to_string())
}

fn random_i64() -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=i32::MAX)
}

fn random_postgresql_timestamp() -> String {
    let mut rng = rand::thread_rng();

    // Define a range for the random timestamp: 2000-01-01 to 2099-12-31
    let start_date =
        NaiveDateTime::parse_from_str("2023-06-03 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let end_date =
        NaiveDateTime::parse_from_str("2023-06-11 23:59:59", "%Y-%m-%d %H:%M:%S").unwrap();

    let start_timestamp = start_date.timestamp();
    let end_timestamp = end_date.timestamp();

    // Generate a random timestamp within the range
    let random_timestamp = rng.gen_range(start_timestamp..=end_timestamp);

    // Convert the timestamp to a DateTime<Utc>
    let random_date =
        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(random_timestamp, 0), Utc);

    // Format the DateTime<Utc> as a string
    random_date.to_rfc3339()
}

pub fn create_table(
    schema_name: &String,
    env: &str,
    api_url: &String,
    access_token: &String,
) -> Result<(), ureq::Error> {
    let resource_name = format!(
        "{}.{}",
        schema_name.to_uppercase(),
        format!("{}_{}", "EVENTS", env.to_uppercase())
    );
    let root = KeyPair::new();
    let public: String = root.public().to_bytes_hex().to_string();
    let private: String = root.private().to_bytes_hex().to_string();
    let biscuit = create_biscuit(resource_name.clone(), root).unwrap();

    let table_config = TableConfig {
        resource_name: resource_name.clone(),
        table_private_key: private,
        table_public_key: public.clone(),
        biscuit: biscuit.to_string(),
    };

    let toml_str = toml::to_string(&table_config).unwrap();
    let mut file = File::create(format!(".flowaim/{}.toml", env))?;
    file.write_all(toml_str.as_bytes())?;

    ureq::post(
        format!(
            "{}/v1/sql/ddl",
            api_url
        )
        .as_str(),
    )
    .set("authorization", format!("Bearer {}", access_token).as_str())
    .set("biscuit", &biscuit)
    .send_json(ureq::json!({ 
        "resourceId": resource_name,
        "sqlText": format!("CREATE TABLE {} (ID INT NOT NULL PRIMARY KEY, PUBLISHER_ID INT, DECK_ID INT, EVENT_TYPE VARCHAR, EVENT_TIMESTAMP TIMESTAMP, PUBLIC_KEY_STRING VARCHAR, SIGNED_TIMESTAMP VARCHAR, TIMESTAMP_UTC_START_SESSION VARCHAR, TIMESTAMP_UTC_END_SESSION VARCHAR) WITH \"template=partitioned,backups=1,public_key={},access_type=public_append\"", resource_name, public) }
    ))?
    .into_string()?;

    Ok(())
}

pub fn insert_rows(
    resource_name: &String,
    api_url: &String,
    access_token: &String,
    biscuit: &String,
    table_rows: &i8,
    publisher_id: &i64,
    deck_id: &i64,
    event_type: &String,
) -> Result<(), ureq::Error> {
    for _n in 1..*table_rows + 1 {
        ureq::post(
            format!(
                "{}/v1/sql/dml",
                api_url
            )
            .as_str(),
        )
        .set("authorization", format!("Bearer {}", access_token).as_str())
        .set("biscuit", &biscuit)
        .send_json(ureq::json!({ 
            "resourceId": resource_name,
            "sqlText": format!("INSERT INTO {} (ID, PUBLISHER_ID, DECK_ID, EVENT_TYPE, EVENT_TIMESTAMP, PUBLIC_KEY_STRING, SIGNED_TIMESTAMP, TIMESTAMP_UTC_START_SESSION, TIMESTAMP_UTC_END_SESSION) VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}')", resource_name, random_i64(), publisher_id, deck_id, event_type, random_postgresql_timestamp(), "0xF4E20531CD11Fb8b70896AA9710FeDbEb9be87c3", "0x5f9e3a4a", "23-03-1990", "19-04-1990") }
        ))?
        .into_string()?;
    }

    Ok(())
}
