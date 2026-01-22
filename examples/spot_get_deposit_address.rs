use mexc_rs::spot::v3::deposit_address::DepositAddressEndpoint;
use mexc_rs::spot::{MexcSpotApiClient, MexcSpotApiEndpoint};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt::init();
    }

    let api_key = std::env::var("MEXC_API_KEY").expect("MEXC_API_KEY not set");
    let secret_key = std::env::var("MEXC_SECRET_KEY").expect("MEXC_SECRET_KEY not set");

    let client = MexcSpotApiClient::new(MexcSpotApiEndpoint::Base)
        .into_with_authentication(api_key, secret_key);

    // Get all deposit addresses for USDT
    let result = client.get_deposit_address("USDT".to_string(), None).await;

    match result {
        Ok(addresses) => {
            tracing::info!("Deposit addresses for USDT:");
            for address in addresses {
                tracing::info!(
                    "  Network: {}, Address: {}, Memo: {:?}",
                    address.network,
                    address.address,
                    address.memo
                );
            }
        }
        Err(e) => {
            tracing::error!("Error: {:?}", e);
        }
    }

    // Get deposit address for USDT on TRC20 network
    let result = client
        .get_deposit_address("USDT".to_string(), Some("TRC20"))
        .await;

    match result {
        Ok(addresses) => {
            tracing::info!("\nDeposit address for USDT on TRC20:");
            for address in addresses {
                tracing::info!(
                    "  Network: {}, Address: {}, Memo: {:?}",
                    address.network,
                    address.address,
                    address.memo
                );
            }
        }
        Err(e) => {
            tracing::error!("Error: {:?}", e);
        }
    }
}
