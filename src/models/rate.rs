use chrono::NaiveDateTime;

use models::*;

#[derive(Debug, Queryable, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rate {
    pub id: ExchangeId,
    pub from: Currency,
    pub to: Currency,
    pub amount: Amount,
    pub expiration: NaiveDateTime,
    pub rate: f64,
    pub amount_currency: Currency,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for Rate {
    fn default() -> Self {
        Self {
            id: ExchangeId::generate(),
            from: Currency::Eth,
            to: Currency::Btc,
            amount: Amount::default(),
            expiration: ::chrono::Utc::now().naive_utc(),
            rate: 0.34343,
            amount_currency: Currency::Eth,
            created_at: ::chrono::Utc::now().naive_utc(),
            updated_at: ::chrono::Utc::now().naive_utc(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRate {
    pub id: ExchangeId,
    pub from: Currency,
    pub to: Currency,
    pub amount: Amount,
    pub amount_currency: Currency,
}

impl Default for GetRate {
    fn default() -> Self {
        Self {
            id: ExchangeId::generate(),
            from: Currency::Eth,
            to: Currency::Btc,
            amount: Amount::default(),
            amount_currency: Currency::Eth,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRate {
    pub exchange_id: ExchangeId,
}

impl Default for RefreshRate {
    fn default() -> Self {
        Self {
            exchange_id: ExchangeId::generate(),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateRefresh {
    pub exchange: Rate,
    pub is_new_rate: bool,
}
