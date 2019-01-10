use chrono::NaiveDateTime;

use validator::Validate;

use models::*;
use schema::transactions_fiat;

#[derive(Debug, Queryable, Clone)]
pub struct TransactionFiat {
    pub id: TransactionId,
    pub fiat_value: Option<String>,
    pub fiat_currency: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for TransactionFiat {
    fn default() -> Self {
        Self {
            id: TransactionId::generate(),
            fiat_value: None,
            fiat_currency: None,
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

impl From<CreateTransaction> for NewTransactionFiat {
    fn from(transaction: CreateTransaction) -> Self {
        Self {
            id: transaction.id,
            fiat_value: transaction.fiat_value,
            fiat_currency: transaction.fiat_currency,
        }
    }
}

#[derive(Debug, Insertable, Validate, Clone)]
#[table_name = "transactions_fiat"]
pub struct NewTransactionFiat {
    pub id: TransactionId,
    pub fiat_value: Option<String>,
    pub fiat_currency: Option<String>,
}

impl Default for NewTransactionFiat {
    fn default() -> Self {
        Self {
            id: TransactionId::generate(),
            fiat_value: None,
            fiat_currency: None,
        }
    }
}
