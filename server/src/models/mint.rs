use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    #[serde(rename = "type")]
    pub type_field: String,
    pub meta_data: MetaData,
    pub id: String,
    pub account_id: String,
    #[serde(rename = "transactionURN")]
    pub transaction_urn: String,
    pub account_ref: AccountRef,
    pub date: String,
    pub description: String,
    pub category: Category,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub match_state: String,
    pub fi_data: FiData,
    pub is_reviewed: bool,
    pub transaction_type: String,
    pub etag: i64,
    pub is_expense: bool,
    pub is_pending: bool,
    pub discretionary_type: String,
    pub is_linked_to_rule: bool,
    pub transaction_review_state: String,
    pub merchant_id: Value,
    pub is_duplicate: Value,
    pub principal: Value,
    pub principal_currency: Value,
    pub interest: Value,
    pub interest_currency: Value,
    pub escrow: Value,
    pub escrow_currency: Value,
    pub parent_id: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaData {
    pub last_updated_date: String,
    pub link: Vec<Link>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub other_attributes: OtherAttributes,
    pub href: String,
    pub rel: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherAttributes {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountRef {
    pub id: i64,
    #[serde(rename = "accountURN")]
    pub account_urn: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub hidden_from_planning_and_trends: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name: String,
    pub category_type: String,
    pub parent_id: String,
    pub parent_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiData {
    pub id: String,
    pub date: String,
    pub amount: f64,
    pub description: String,
    pub inferred_description: String,
    pub inferred_category: InferredCategory,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InferredCategory {
    pub id: String,
    pub name: String,
}
