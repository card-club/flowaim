extern crate serde;
extern crate serde_json;
extern crate tabled;
extern crate ureq;

use serde::Deserialize;
use std::cmp::Ordering;
use tabled::{Table, Tabled};

#[derive(Deserialize, Tabled)]
struct EventCount {
    date: String,
    event_type: String,
    count: i32,
}

pub fn print_stats(
    resource_name: &String,
    api_url: &String,
    access_token: &String,
    biscuit: &String,
) -> Result<(), ureq::Error> {
    let response = ureq::post(
        format!(
            "{}/v1/sql/dql",
            api_url
        )
        .as_str(),
    )
    .set("authorization", format!("Bearer {}", access_token).as_str())
    .set("biscuit", &biscuit)
    .send_json(ureq::json!({ 
        "resourceId": resource_name,
        "sqlText": format!("SELECT DATE(event_timestamp) AS date, event_type, COUNT(*) as count FROM {} WHERE event_timestamp >= NOW() - INTERVAL '7 days' GROUP BY date, event_type ORDER BY date, event_type;", resource_name) }
    ))?
    .into_string()?;

    let mut events: Vec<EventCount> =
        serde_json::from_str(response.as_str()).expect("Failed to parse JSON response");

    events.sort_by(|a, b| {
        let date_order = a.date.cmp(&b.date);
        if date_order == Ordering::Equal {
            match (a.event_type.as_str(), b.event_type.as_str()) {
                ("deck_start", _) => Ordering::Less,
                (_, "deck_start") => Ordering::Greater,
                ("deck_end", _) => Ordering::Less,
                (_, "deck_end") => Ordering::Greater,
                ("ad_view", _) => Ordering::Less,
                (_, "ad_view") => Ordering::Greater,
                ("ad_click", _) => Ordering::Greater,
                (_, "ad_click") => Ordering::Less,
                (_, _) => Ordering::Equal,
            }
        } else {
            date_order
        }
    });

    println!("\nEvents in the last 7 days:");
    let table = Table::new(&events);
    println!("{}", table);
    Ok(())
}

// SELECT event_type, COUNT(*) as count FROM events WHERE date >= NOW() - INTERVAL '7 days' GROUP BY event_type;
