use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use diesel::sqlite::SqliteValue;
use serde::{Deserialize, Serialize};

use crate::db::model::account::AccountId;
use crate::db::model::user::UserId;
use crate::db::schema::transactions;

#[repr(i32)]
#[derive(AsExpression, Debug, Clone, Copy, Serialize, Deserialize, FromSqlRow)]
#[diesel(sql_type = Integer)]
pub enum TransactionType {
    Buy = 0,
    Sell = 1,
}

impl FromSql<Text, diesel::sqlite::Sqlite> for TransactionType {
    fn from_sql(bytes: SqliteValue) -> deserialize::Result<Self> {
        let value =
            <String as deserialize::FromSql<Text, diesel::sqlite::Sqlite>>::from_sql(bytes)?;
        match value.as_str() {
            "buy" => Ok(TransactionType::Buy),
            "sell" => Ok(TransactionType::Sell),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl FromSql<Integer, diesel::sqlite::Sqlite> for TransactionType {
    fn from_sql(bytes: SqliteValue) -> deserialize::Result<Self> {
        let value =
            <i32 as deserialize::FromSql<Integer, diesel::sqlite::Sqlite>>::from_sql(bytes)?;
        match value {
            0 => Ok(TransactionType::Buy),
            1 => Ok(TransactionType::Sell),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = transactions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Transaction {
    pub id: Option<i32>,
    /// Reference to the user who owns this transaction.
    pub owner_id: UserId,
    /// Reference to the account associated with this transaction.
    pub account_id: AccountId,
    /// Whether the transaction is a buy or sell.
    pub transaction_type: TransactionType,
    pub ticker_symbol: String,
    pub transaction_date: String,
    pub quantity: f32,
    pub price_per_share: f32,
    pub fees: f32,
    pub created_at: String,
    pub updated_at: String,
}
