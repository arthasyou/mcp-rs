use mcp_tools_rs::tools::web_search::do_web_search;

#[tokio::main]
async fn main() {
    let query = "Rust programming language";
    match do_web_search(query).await {
        Ok(result) => {
            println!(
                "Search result JSON:\n{}",
                serde_json::to_string_pretty(&result).unwrap()
            );
        }
        Err(e) => {
            eprintln!("Error performing web search: {}", e);
        }
    }
}
