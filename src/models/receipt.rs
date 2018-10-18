use uuid::Uuid;

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ReceiptType {
    Account,
    Address,
}

#[derive(Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Receipt(String);

impl Default for Receipt {
    fn default() -> Self {
        Receipt(Uuid::new_v4().to_string())
    }
}
