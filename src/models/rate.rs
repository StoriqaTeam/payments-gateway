use std::time::SystemTime;

use models::*;

#[derive(Debug, Queryable, Clone, Deserialize)]
pub struct Rate {
    pub id: ExchangeId,
    pub from: Currency,
    pub to: Currency,
    pub amount: Amount,
    pub expiration: SystemTime,
    pub rate: f64,
    pub amount_currency: Currency,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Default for Rate {
    fn default() -> Self {
        Self {
            id: ExchangeId::generate(),
            from: Currency::Eth,
            to: Currency::Btc,
            amount: Amount::default(),
            expiration: SystemTime::now(),
            rate: 0.34343,
            amount_currency: Currency::Eth,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
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
