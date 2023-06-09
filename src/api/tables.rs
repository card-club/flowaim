use biscuit_auth::{error, macros::*, KeyPair};

fn create_biscuit(resource_name: String, root: KeyPair) -> Result<String, error::Token> {
    let schema_table_name = resource_name.to_lowercase();
    let biscuit = biscuit!(r#"
        sxt:capability("ddl_create", {schema_table_name});
        sxt:capability("ddl_alter", {schema_table_name});
        sxt:capability("ddl_drop", {schema_table_name});
        sxt:capability("dml_insert", {schema_table_name});
        sxt:capability("dql_select", {schema_table_name});
        sxt:capability("kafka_icm_create", {schema_table_name});
        sxt:capability("kafka_icm_read", {schema_table_name});
        sxt:capability("kafka_icm_update", {schema_table_name});
    "#).build(&root)?;

    Ok(biscuit.to_base64()?.to_string())
}

pub fn create_table(schema_name : &String, env : &str, api_url: &String, access_token: &String) -> Result<(), ureq::Error> {
    let resource_name = format!("{}.{}", schema_name.to_uppercase(), format!("{}_{}", "EVENTS", env.to_uppercase()));
    println!("Resource Name: {}", resource_name );
    let root = KeyPair::new();
    let public: String = root.public().to_bytes_hex().to_string();
    // let private: String = root.private().to_bytes_hex().to_string();
    let biscuit = create_biscuit(resource_name.clone(), root).unwrap();
    println!("Biscuit: {}", biscuit);
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
        "sqlText": format!("CREATE TABLE {} (ID INT NOT NULL PRIMARY KEY, PUBLISHER_ID INT, DECK_ID INT, EVENT_TYPE VARCHAR, PUBLIC_KEY_STRING VARCHAR, SIGNED_TIMESTAMP VARCHAR, TIMESTAMP_UTC_START_SESSION VARCHAR, TIMESTAMP_UTC_END_SESSION VARCHAR) WITH \"template=partitioned,backups=1,public_key={},access_type=public_append\"", resource_name, public) }
    ))?
    .into_string()?;

    Ok(())
}

pub fn insert_rows(schema_name : &String, env : &str, api_url: &String, access_token: &String, biscuit: &String, table_rows: &i8, publisher_id: &i64, deck_id: &i64, event_type: &String) -> Result<(), ureq::Error> {
    let resource_name = format!("{}.{}", schema_name, format!("{}_{}", "EVENTS", env.to_uppercase()));
    // for loop to insert multiple rows, should be fixed with autoincrement id in the future
    for n in 1..*table_rows + 1 {
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
            "sqlText": format!("INSERT INTO {} (ID, PUBLISHER_ID, DECK_ID, EVENT_TYPE, PUBLIC_KEY_STRING, SIGNED_TIMESTAMP, TIMESTAMP_UTC_START_SESSION, TIMESTAMP_UTC_END_SESSION) VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}')", resource_name.to_uppercase(), n, publisher_id, deck_id, event_type, "0xF4E20531CD11Fb8b70896AA9710FeDbEb9be87c3", "0x5f9e3a4a", "23-03-1990", "19-04-1990") }
        ))?
        .into_string()?;
    }
    
    Ok(())
}

