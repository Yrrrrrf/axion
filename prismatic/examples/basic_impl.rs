use reqwest::Client;
use std::time::Duration;
use std::time::Instant;


// todo: ON PRISMATIC
// todo: - Add an improved ping
// todo: - Add a timeout to the request

async fn check_website_status(url: &str, timeout_secs: u64) {
    println!("Checking: {}", url);

    // Start timer
    let start = Instant::now();

    // Create client with timeout
    let client = match Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            println!("  Error: Failed to create client - {}", e);
            println!("  Elapsed: 0.00 ms (failed at client creation)\n");
            return;
        }
    };

    // Make the request
    match client.get(url).send().await {
        Ok(response) => {
            let duration_ms = start.elapsed().as_millis();
            let status = response.status();
            println!("  Response Time: {} ms", duration_ms);
            println!(
                "  Status: {} ({})",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            );
        }
        Err(e) => {
            let duration_ms = start.elapsed().as_millis();
            println!("  Response Time: {} ms (to error)", duration_ms);
            println!("  Error: Request failed - {}", e);
        }
    }
    println!();
}

#[tokio::main]
async fn main() {
    println!("Website Status Checker\n");

    let websites = vec![
        "https://www.google.com",
        "https://www.github.com",
        "https://www.rust-lang.org",
        "https://httpstat.us/404",
        "http://httpbin.org/delay/1",
        "https://nonexistent-site-for-testing-12345.org",
        "invalid-url-format",
    ];

    let total_start = Instant::now();

    for url in websites {
        check_website_status(url, 5).await;
    }

    println!(
        "Total time elapsed: {} ms",
        total_start.elapsed().as_millis()
    );
}
