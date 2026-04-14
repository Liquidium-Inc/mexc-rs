use crate::spot::v3::{ApiError, ApiResponse, ApiResult, ErrorCode, ErrorResponse};
use crate::spot::MexcSpotApiClientWithAuthentication;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::de::{self, Visitor};
use std::fmt::{Display, Formatter};

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawOutput {
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WithdrawStatus {
    Apply = 1,
    Auditing = 2,
    Wait = 3,
    Processing = 4,
    WaitPackaging = 5,
    WaitConfirm = 6,
    Success = 7,
    Failed = 8,
    Cancel = 9,
    Manual = 10,
}

impl WithdrawStatus {
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            1 => Some(Self::Apply),
            2 => Some(Self::Auditing),
            3 => Some(Self::Wait),
            4 => Some(Self::Processing),
            5 => Some(Self::WaitPackaging),
            6 => Some(Self::WaitConfirm),
            7 => Some(Self::Success),
            8 => Some(Self::Failed),
            9 => Some(Self::Cancel),
            10 => Some(Self::Manual),
            _ => None,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_uppercase().as_str() {
            "APPLY" => Some(Self::Apply),
            "AUDITING" => Some(Self::Auditing),
            "WAIT" => Some(Self::Wait),
            "PROCESSING" => Some(Self::Processing),
            "WAIT_PACKAGING" | "WAITPACKAGING" => Some(Self::WaitPackaging),
            "WAIT_CONFIRM" | "WAITCONFIRM" => Some(Self::WaitConfirm),
            "SUCCESS" => Some(Self::Success),
            "FAILED" => Some(Self::Failed),
            "CANCEL" => Some(Self::Cancel),
            "MANUAL" => Some(Self::Manual),
            _ => None,
        }
    }

    pub const fn code(self) -> i32 {
        self as i32
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Apply => "APPLY",
            Self::Auditing => "AUDITING",
            Self::Wait => "WAIT",
            Self::Processing => "PROCESSING",
            Self::WaitPackaging => "WAIT_PACKAGING",
            Self::WaitConfirm => "WAIT_CONFIRM",
            Self::Success => "SUCCESS",
            Self::Failed => "FAILED",
            Self::Cancel => "CANCEL",
            Self::Manual => "MANUAL",
        }
    }
}

impl Display for WithdrawStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl serde::Serialize for WithdrawStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(self.code())
    }
}

impl<'de> serde::Deserialize<'de> for WithdrawStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct WithdrawStatusVisitor;

        impl<'de> Visitor<'de> for WithdrawStatusVisitor {
            type Value = WithdrawStatus;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("withdraw status code as integer/string or status name")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let value = i32::try_from(value)
                    .map_err(|_| de::Error::custom("withdraw status code out of i32 range"))?;
                WithdrawStatus::from_code(value)
                    .ok_or_else(|| de::Error::custom(format!("unknown withdraw status code: {value}")))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let value = i32::try_from(value)
                    .map_err(|_| de::Error::custom("withdraw status code out of i32 range"))?;
                WithdrawStatus::from_code(value)
                    .ok_or_else(|| de::Error::custom(format!("unknown withdraw status code: {value}")))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if let Ok(code) = value.parse::<i32>() {
                    return WithdrawStatus::from_code(code).ok_or_else(|| {
                        de::Error::custom(format!("unknown withdraw status code: {code}"))
                    });
                }

                WithdrawStatus::from_name(value).ok_or_else(|| {
                    de::Error::custom(format!("unknown withdraw status name: {value}"))
                })
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&value)
            }
        }

        deserializer.deserialize_any(WithdrawStatusVisitor)
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawHistoryRecord {
    pub address: String,
    #[serde(deserialize_with = "crate::spot::v3::deserialize_string_from_number")]
    pub amount: String,
    #[serde(deserialize_with = "crate::spot::v3::deserialize_string_from_number")]
    pub apply_time: String,
    pub coin: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdraw_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(deserialize_with = "crate::spot::v3::deserialize_string_from_number")]
    pub transfer_type: String,
    pub status: WithdrawStatus,
    #[serde(deserialize_with = "crate::spot::v3::deserialize_string_from_number")]
    pub transaction_fee: String,
    #[serde(default, deserialize_with = "crate::spot::v3::deserialize_option_i32_from_string_or_number")]
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
    use crate::spot::v3::ApiResponse;

    #[test]
    fn withdraw_history_deserializes_numeric_and_string_fields() {
        let json = r#"
        [
          {
            "address": "0x123",
            "amount": 8.91,
            "applyTime": 1754936662000,
            "coin": "USDC",
            "id": "321",
            "network": "ERC20",
            "transferType": 0,
            "status": "6",
            "transactionFee": "4.4",
            "confirmNo": "2",
            "txId": "0xabc"
          }
        ]
        "#;

        let response = serde_json::from_str::<ApiResponse<Vec<WithdrawHistoryRecord>>>(json)
            .expect("withdraw history should deserialize");
        let records = response.into_result().expect("success response expected");

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].amount, "8.91");
        assert_eq!(records[0].apply_time, "1754936662000");
        assert_eq!(records[0].transfer_type, "0");
        assert_eq!(records[0].status, WithdrawStatus::WaitConfirm);
        assert_eq!(records[0].confirm_no, Some(2));
    }

    #[test]
    fn withdraw_status_map_is_correct() {
        assert_eq!(WithdrawStatus::from_code(1), Some(WithdrawStatus::Apply));
        assert_eq!(WithdrawStatus::from_code(2), Some(WithdrawStatus::Auditing));
        assert_eq!(WithdrawStatus::from_code(3), Some(WithdrawStatus::Wait));
        assert_eq!(WithdrawStatus::from_code(4), Some(WithdrawStatus::Processing));
        assert_eq!(
            WithdrawStatus::from_code(5),
            Some(WithdrawStatus::WaitPackaging)
        );
        assert_eq!(WithdrawStatus::from_code(6), Some(WithdrawStatus::WaitConfirm));
        assert_eq!(WithdrawStatus::from_code(7), Some(WithdrawStatus::Success));
        assert_eq!(WithdrawStatus::from_code(8), Some(WithdrawStatus::Failed));
        assert_eq!(WithdrawStatus::from_code(9), Some(WithdrawStatus::Cancel));
        assert_eq!(WithdrawStatus::from_code(10), Some(WithdrawStatus::Manual));
    }

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
