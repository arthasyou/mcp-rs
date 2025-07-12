use reqwest::Client;
use scraper::{Html, Selector};
use serde::Serialize;
use serde_json::Value;
use urlencoding::decode;

use crate::error::{Error, Result};

#[derive(Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
}

/// 实际执行 web 搜索逻辑
pub async fn do_web_search(query: &str) -> Result<Value> {
    let client = Client::new();
    let url = format!("https://html.duckduckgo.com/html/?q={}", query);
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| Error::System(e.to_string()))?;
    let body = res.text().await.map_err(|e| Error::System(e.to_string()))?;

    let doc = Html::parse_document(&body);
    let selector = Selector::parse("a.result__a").map_err(|e| Error::System(e.to_string()))?;

    let mut results = Vec::new();
    for element in doc.select(&selector).take(5) {
        let title = element
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();
        let raw_url = element.value().attr("href").unwrap_or("").to_string();
        let url = if let Some(uddg) = raw_url.split("uddg=").nth(1) {
            decode(uddg).unwrap_or_else(|_| "".into()).to_string()
        } else {
            "".to_string()
        };

        // 请求目标页面并提取正文
        let mut content = String::new();
        if let Ok(page_res) = client.get(&url).send().await {
            if let Ok(page_html) = page_res.text().await {
                let page_doc = Html::parse_document(&page_html);
                if let Ok(p_selector) = Selector::parse("p") {
                    for p in page_doc.select(&p_selector).take(10) {
                        let text = p
                            .text()
                            .collect::<Vec<_>>()
                            .join(" ")
                            .replace('\n', " ")
                            .replace('\u{00a0}', " ")
                            .trim()
                            .to_string();
                        if !text.is_empty() {
                            content.push_str(&text);
                            content.push('\n');
                        }
                    }
                }
            }
        }

        if content.len() > 1000 {
            content.truncate(1000);
        }

        results.push(SearchResult {
            title,
            url,
            content,
        });
    }

    let json = serde_json::to_value(&results)?;
    Ok(json)
}
