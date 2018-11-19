use chrono::NaiveDateTime;

use validator::Validate;

use models::*;
use schema::accounts;

#[derive(Debug, Queryable, Clone)]
pub struct Account {
    pub id: AccountId,
    pub user_id: UserId,
    pub currency: Currency,
    pub account_address: AccountAddress,
    pub name: String,
    pub balance: Amount,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub callback_url: Option<String>,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            id: AccountId::generate(),
            user_id: UserId::generate(),
            currency: Currency::Eth,
            account_address: AccountAddress::default(),
            name: "new acc".to_string(),
            balance: Amount::default(),
            created_at: ::chrono::Utc::now().naive_utc(),
            updated_at: ::chrono::Utc::now().naive_utc(),
            callback_url: None,
        }
    }
}

impl From<NewAccount> for Account {
    fn from(new_account: NewAccount) -> Self {
        Self {
            id: new_account.id,
            name: new_account.name,
            user_id: new_account.user_id,
            currency: new_account.currency,
            account_address: new_account.account_address,
            ..Default::default()
        }
    }
}

#[derive(Debug, Insertable, Validate, Clone)]
#[table_name = "accounts"]
pub struct NewAccount {
    pub id: AccountId,
    pub user_id: UserId,
    pub currency: Currency,
    #[validate]
    pub account_address: AccountAddress,
    #[validate(length(min = "1", max = "40", message = "Name must not be empty "))]
    pub name: String,
    pub callback_url: Option<String>,
}

impl Default for NewAccount {
    fn default() -> Self {
        Self {
            id: AccountId::generate(),
            name: "new acc".to_string(),
            user_id: UserId::generate(),
            currency: Currency::Eth,
            account_address: AccountAddress::default(),
            callback_url: None,
        }
    }
}

#[derive(Debug, Insertable, Validate, AsChangeset, Clone, Default, Serialize)]
#[table_name = "accounts"]
pub struct UpdateAccount {
    #[validate(length(min = "1", max = "40", message = "Name must not be empty "))]
    pub name: String,
}

#[derive(Debug, Clone, Validate)]
pub struct CreateAccount {
    pub id: AccountId,
    pub user_id: UserId,
    pub currency: Currency,
    #[validate(length(min = "1", max = "40", message = "Name must not be empty "))]
    pub name: String,
    pub callback_url: Option<String>,
}

impl Default for CreateAccount {
    fn default() -> Self {
        Self {
            id: AccountId::generate(),
            user_id: UserId::generate(),
            currency: Currency::Eth,
            name: String::default(),
            callback_url: None,
        }
    }
}

impl From<(CreateAccount, AccountAddress)> for NewAccount {
    fn from(acc: (CreateAccount, AccountAddress)) -> Self {
        Self {
            id: acc.0.id,
            user_id: acc.0.user_id,
            currency: acc.0.currency,
            account_address: acc.1,
            name: acc.0.name,
            callback_url: acc.0.callback_url,
        }
    }
}
