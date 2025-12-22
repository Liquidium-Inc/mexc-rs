use mexc_rs::spot::v3::withdraw::{WithdrawEndpoint, WithdrawRequest};
use mexc_rs::spot::{MexcSpotApiClient, MexcSpotApiEndpoint};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
    let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");

    let client = MexcSpotApiClient::new(MexcSpotApiEndpoint::Base)
        .into_with_authentication(api_key, secret_key);

    // WARNING: This example performs a real withdrawal!
    // Make sure to update the parameters below with your actual withdrawal details
    let request = WithdrawRequest {
        coin: "USDT".to_string(),
        withdraw_order_id: None, // Optional custom order ID
        network: Some("TRC20".to_string()), // Specify network (TRC20, ERC20, BEP20, etc.)
        address: "YOUR_WITHDRAWAL_ADDRESS_HERE".to_string(), // IMPORTANT: Replace with actual address
        memo: None, // Some coins like EOS, XRP require a memo/tag
        amount: "10.0".to_string(), // Amount to withdraw
        remark: Some("Withdrawal from Rust client".to_string()), // Optional remark
    };

    println!("Submitting withdrawal request...");
    println!("  Coin: {}", request.coin);
    println!("  Network: {:?}", request.network);
    println!("  Address: {}", request.address);
    println!("  Amount: {}", request.amount);
    println!();

    let result = client.withdraw(request).await;

    match result {
        Ok(output) => {
            println!("Withdrawal submitted successfully!");
            println!("  Withdrawal ID: {}", output.id);
        }
        Err(e) => {
            eprintln!("Error submitting withdrawal: {:?}", e);
        }
    }
}
