use crate::spot::v3::{ApiError, ApiResponse, ApiResult, ErrorCode, ErrorResponse};
use crate::spot::MexcSpotApiClientWithAuthentication;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawOutput {
    pub id: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawHistoryRecord {
    pub address: String,
    pub amount: String,
    pub apply_time: String,
    pub coin: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdraw_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    pub transfer_type: String,
    pub status: String,
    pub transaction_fee: String,
    pub confirm_no: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trans_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coin_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcoin_id: Option<String>,
}

#[async_trait]
pub trait WithdrawEndpoint {
    async fn withdraw(&self, request: WithdrawRequest) -> ApiResult<WithdrawOutput>;
    async fn withdraw_history(
        &self,
        request: WithdrawHistoryRequest,
    ) -> ApiResult<Vec<WithdrawHistoryRecord>>;
}

#[derive(Debug, Clone)]
pub struct WithdrawRequest {
    pub coin: String,
    pub withdraw_order_id: Option<String>,
    pub network: Option<String>,
    pub address: String,
    pub memo: Option<String>,
    pub amount: String,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct WithdrawHistoryRequest {
    /// Specific cryptocurrency (optional)
    pub coin: Option<String>,
    /// Withdrawal status filter (optional)
    pub status: Option<String>,
    /// Default: 1000, Max: 1000
    pub limit: Option<u32>,
    /// Default: 7 days ago from current time (in milliseconds)
    pub start_time: Option<i64>,
    /// Default: current time (in milliseconds)
    pub end_time: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct WithdrawQuery {
    pub coin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdraw_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_work: Option<String>,
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    pub recv_window: Option<u64>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct WithdrawHistoryQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
    pub recv_window: Option<u64>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

#[async_trait]
impl WithdrawEndpoint for MexcSpotApiClientWithAuthentication {
    async fn withdraw(&self, request: WithdrawRequest) -> ApiResult<WithdrawOutput> {
        let endpoint = format!("{}/api/v3/capital/withdraw", self.endpoint.as_ref());
        let query = self.sign_query(WithdrawQuery {
            coin: request.coin,
            withdraw_order_id: request.withdraw_order_id,
            net_work: request.network,
            address: request.address,
            memo: request.memo,
            amount: request.amount,
            remark: request.remark,
            recv_window: None,
            timestamp: Utc::now(),
        })?;
        let response = self
            .reqwest_client
            .post(endpoint)
            .query(&query)
            .send()
            .await?;

        // Read entire body as text first
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            // Try to parse the real MEXC error
            let err =
                serde_json::from_str::<ErrorResponse>(&body).unwrap_or_else(|_| ErrorResponse {
                    raw_code: 999,
                    code: ErrorCode::InternalError,
                    _extend: None,
                    msg: format!("HTTP {status}, body: {body}"),
                });

            return Err(err.into());
        }

        // Success path: parse as normal APIResponse
        let api_response: ApiResponse<WithdrawOutput> =
            serde_json::from_str(&body).map_err(ApiError::from)?;

        let output = api_response.into_api_result()?;
        Ok(output)
    }

    async fn withdraw_history(
        &self,
        request: WithdrawHistoryRequest,
    ) -> ApiResult<Vec<WithdrawHistoryRecord>> {
        let endpoint = format!("{}/api/v3/capital/withdraw/history", self.endpoint.as_ref());
        let query = self.sign_query(WithdrawHistoryQuery {
            coin: request.coin,
            status: request.status,
            limit: request.limit,
            start_time: request.start_time,
            end_time: request.end_time,
            recv_window: None,
            timestamp: Utc::now(),
        })?;
        let response = self
            .reqwest_client
            .get(endpoint)
            .query(&query)
            .send()
            .await?;
        println!("{:?} ", response);

        let api_response = response
            .json::<ApiResponse<Vec<WithdrawHistoryRecord>>>()
            .await?;
        let output = api_response.into_api_result()?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Ignored by default as this performs real withdrawals
    async fn test_withdraw() {
        let client = MexcSpotApiClientWithAuthentication::new_for_test();
        let request = WithdrawRequest {
            coin: "USDT".to_string(),
            withdraw_order_id: None,
            network: Some("TRC20".to_string()),
            address: "TXobiKkdciupZrhdvZyTSSLjE8CmZAufS".to_string(),
            memo: None,
            amount: "10".to_string(),
            remark: Some("Test withdrawal".to_string()),
        };
        let result = client.withdraw(request).await;
        assert!(result.is_ok());
    }
}
