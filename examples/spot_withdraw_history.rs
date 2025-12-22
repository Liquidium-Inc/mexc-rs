use mexc_rs::spot::v3::withdraw::{WithdrawEndpoint, WithdrawHistoryRequest};
use mexc_rs::spot::{MexcSpotApiClient, MexcSpotApiEndpoint};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
    let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");

    let client = MexcSpotApiClient::new(MexcSpotApiEndpoint::Base)
        .into_with_authentication(api_key, secret_key);

    // Query withdraw history
    // This example retrieves the last 7 days of withdrawal history (default)
    let request = WithdrawHistoryRequest {
        coin: Some("USDT".to_string()), // Optional: filter by specific coin
        status: None,                    // Optional: filter by status
        limit: Some(100),                // Optional: limit results (max 1000, default 1000)
        start_time: None,                // Optional: start time in milliseconds (default: 7 days ago)
        end_time: None,                  // Optional: end time in milliseconds (default: current time)
    };

    println!("Querying withdrawal history...");
    if let Some(coin) = &request.coin {
        println!("  Filtering by coin: {}", coin);
    }
    if let Some(limit) = request.limit {
        println!("  Limit: {}", limit);
    }
    println!();

    let result = client.withdraw_history(request).await;

    match result {
        Ok(records) => {
            println!("Found {} withdrawal records:", records.len());
            println!();

            for (index, record) in records.iter().enumerate() {
                println!("Record #{}:", index + 1);
                println!("  ID: {}", record.id);
                println!("  Coin: {}", record.coin);
                println!("  Amount: {}", record.amount);
                println!("  Status: {}", record.status);
                println!("  Address: {}", record.address);
                println!("  Network: {:?}", record.network);
                println!("  Transaction Fee: {}", record.transaction_fee);
                println!("  Apply Time: {}", record.apply_time);
                if let Some(tx_id) = &record.tx_id {
                    println!("  Transaction ID: {}", tx_id);
                }
                if let Some(trans_hash) = &record.trans_hash {
                    println!("  Transaction Hash: {}", trans_hash);
                }
                if let Some(remark) = &record.remark {
                    println!("  Remark: {}", remark);
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error querying withdrawal history: {:?}", e);
        }
    }
}
