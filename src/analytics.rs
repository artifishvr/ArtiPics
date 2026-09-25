use std::collections::HashMap;

use axum::http::{HeaderMap, HeaderValue, header};
use reqwest::header as reqwest_header;

use crate::reqwest_client::get_client;


pub async fn yell(route: &str, headers: HeaderMap) -> Result<(), reqwest::Error> {
    let default_header = HeaderValue::from_static("meow");

    let user_agent = headers
        .get(header::USER_AGENT)
        .cloned()
        .unwrap_or(default_header.clone());
    let ip = headers
        .get("CF-Connecting-IP")
        .cloned()
        .unwrap_or(default_header);


    let analytics_url = format!("app://localhost{}", route);

    let mut analytics_body = HashMap::new();
    analytics_body.insert("name", "Ping");
    analytics_body.insert("url", &analytics_url);
    analytics_body.insert("domain", "artiapartment.vrc.bz");

    let mut analytics_headers = header::HeaderMap::new();
    analytics_headers.insert(
        reqwest::header::HeaderName::from_static("x-plausible-ip"),
        ip,
    );
    analytics_headers.insert(reqwest_header::USER_AGENT, user_agent);

    let res = get_client()
        .post("https://a.arti.lol/api/event")
        .json(&analytics_body)
        .headers(analytics_headers)
        .send()
        .await?
        .text()
        .await?;

    println!("analytics says: {}", res);

    Ok(())
}
