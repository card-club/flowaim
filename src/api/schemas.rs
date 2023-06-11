pub fn create_schema(
    schema_name: &String,
    api_url: &String,
    access_token: &String,
) -> Result<(), ureq::Error> {
    ureq::post(format!("{}/v1/sql/ddl", api_url).as_str())
        .set("authorization", format!("Bearer {}", access_token).as_str())
        .send_json(ureq::json!({
            "sqlText": format!("CREATE SCHEMA {}", schema_name)
        }))?
        .into_string()?;

    Ok(())
}

pub fn drop_schema(
    schema_name: &String,
    api_url: &String,
    access_token: &String,
) -> Result<(), ureq::Error> {
    ureq::post(format!("{}/v1/sql/ddl", api_url).as_str())
        .set("authorization", format!("Bearer {}", access_token).as_str())
        .send_json(ureq::json!({
            "sqlText": format!("DROP SCHEMA {}", schema_name)
        }))?
        .into_string()?;

    Ok(())
}
