use diesel::prelude::*;
use chrono::prelude::*;

use super::schema::passwords;

use macroman_tools::MacRomanString;

#[derive(Queryable)]
pub struct Password {
    pub id: i32,
    pub password: String,
    pub notes: String,
    pub created_at: String,
}

#[derive(Insertable)]
#[table_name="passwords"]
struct NewPasswordEntry {
    password: String,
    notes: String,
    created_at: String,
}

impl Password {
    pub fn is_authorized(db: &SqliteConnection, provided_password: &MacRomanString<255>) -> Result<bool, Box<dyn std::error::Error>> {
        use crate::schema::passwords::dsl::*;

        let provided_password = provided_password.as_string();

        let results = passwords.filter(password.eq(provided_password))
            .limit(1)
            .load::<Password>(db)?;

        Ok(results.len() == 1)
    }
}
