use crate::spot::v3::{ApiResponse, ApiResult};
use crate::spot::MexcSpotApiClientWithAuthentication;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddress {
    pub coin: String,
    pub network: String,
    pub address: String,
    pub memo: Option<String>,
}

#[async_trait]
pub trait DepositAddressEndpoint {
    async fn get_deposit_address(
        &self,
        coin: String,
        network: Option<&str>,
    ) -> ApiResult<Vec<DepositAddress>>;
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddressQuery {
    pub coin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    pub recv_window: Option<u64>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

#[async_trait]
impl DepositAddressEndpoint for MexcSpotApiClientWithAuthentication {
    async fn get_deposit_address(
        &self,
        coin: String,
        network: Option<&str>,
    ) -> ApiResult<Vec<DepositAddress>> {
        let endpoint = format!("{}/api/v3/capital/deposit/address", self.endpoint.as_ref());
        let query = self.sign_query(DepositAddressQuery {
            coin,
            network: network.map(|d| d.to_string()),
            recv_window: None,
            timestamp: Utc::now(),
        })?;
        let response = self
            .reqwest_client
            .get(endpoint)
            .query(&query)
            .send()
            .await?;
        let api_response = response
            .json::<ApiResponse<Vec<DepositAddress>>>()
            .await?;
        let output = api_response.into_api_result()?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_deposit_address() {
        let client = MexcSpotApiClientWithAuthentication::new_for_test();
        let result = client.get_deposit_address("USDT".to_string(), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_deposit_address_with_network() {
        let client = MexcSpotApiClientWithAuthentication::new_for_test();
        let result = client
            .get_deposit_address("USDT".to_string(), Some("TRC20".to_string()))
            .await;
        assert!(result.is_ok());
    }
}
