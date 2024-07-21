use core::result::Result as ResultClient;
use reqwest::{header, Response as ResponseClient};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::{future::IntoFuture, str::Split};

use worker::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ContributionInfo {
    calendar: String,
    #[serde(rename = "textContributions")]
    text_contributions: String,
}

#[event(fetch)]
async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let path: String = req.path();
    let mut split_path: Split<&str> = path.split("/");
    let username: &str = split_path.nth(1).unwrap_or_default();
    let year: &str = split_path.next().unwrap_or_default();

    let contributions_info: ContributionInfo = get_contributions_info(username, year)
        .into_future()
        .await
        .unwrap();

    let mut headers: Headers = Headers::default();
    let _ = headers.set(header::ACCESS_CONTROL_ALLOW_ORIGIN.as_str(), "*");
    let _ = headers.set(
        header::ACCESS_CONTROL_ALLOW_METHODS.as_str(),
        "GET, POST, PUT, DELETE",
    );
    let _ = headers.set(
        header::ACCESS_CONTROL_ALLOW_HEADERS.as_str(),
        "Content-Type, Authorization",
    );

    Response::builder()
        .with_headers(headers)
        .from_json(&contributions_info)
}

async fn get_contributions_info(
    username: &str,
    year: &str,
) -> ResultClient<ContributionInfo, Box<dyn std::error::Error>> {
    let url: String = format!(
        "https://github.com/users/{}/contributions?from={}-01-01&to={}-12-31",
        username, year, year
    );

    let res: ResponseClient = reqwest::Client::new().get(url).send().await?;

    let html: String = res.text().await?;

    let fragment: Html = Html::parse_document(&html);

    let selector_contributions: Selector = Selector::parse(".f4").unwrap();
    let selector_calendar: Selector = Selector::parse(".graph-canvas").unwrap();

    let input_contributions: ElementRef = fragment.select(&selector_contributions).next().unwrap();
    let input_calendar: ElementRef = fragment.select(&selector_calendar).next().unwrap();

    let text_contributions: String = input_contributions
        .text()
        .collect::<String>()
        .trim()
        .to_owned();

    let result: ContributionInfo = ContributionInfo {
        calendar: input_calendar.inner_html(),
        text_contributions,
    };

    Ok(result)
}
