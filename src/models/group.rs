use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
}

impl<'c> FromRow<'c, PgRow> for Group {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Group {
            id: row.get(0),
            name: row.get(1),
        })
    }
}
