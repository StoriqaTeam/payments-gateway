use chrono::NaiveDateTime;

use validator::Validate;

use models::*;
use schema::transactions_fiat;

#[derive(Debug, Queryable, Clone)]
pub struct TransactionFiat {
    pub id: TransactionId,
    pub fiat_value: String,
    pub fiat_currency: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for TransactionFiat {
    fn default() -> Self {
        Self {
            id: TransactionId::generate(),
            fiat_value: "123".to_string(),
            fiat_currency: "USD".to_string(),
            created_at: ::chrono::Utc::now().naive_utc(),
            updated_at: ::chrono::Utc::now().naive_utc(),
        }
    }
}

impl From<NewTransactionFiat> for TransactionFiat {
    fn from(new_transaction: NewTransactionFiat) -> Self {
        Self {
            id: new_transaction.id,
            fiat_value: new_transaction.fiat_value,
            fiat_currency: new_transaction.fiat_currency,
            ..Default::default()
        }
    }
}

impl NewTransactionFiat {
    pub fn new(id: TransactionId, fiat_value: String, fiat_currency: String) -> Self {
        Self {
            id,
            fiat_value,
            fiat_currency,
        }
    }
}

#[derive(Debug, Insertable, Validate, Clone)]
#[table_name = "transactions_fiat"]
pub struct NewTransactionFiat {
    pub id: TransactionId,
    pub fiat_value: String,
    pub fiat_currency: String,
}

impl Default for NewTransactionFiat {
    fn default() -> Self {
        Self {
            id: TransactionId::generate(),
            fiat_value: "123".to_string(),
            fiat_currency: "USD".to_string(),
        }
    }
}
