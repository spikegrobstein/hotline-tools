use diesel::prelude::*;

use crate::util::now;

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
#[table_name = "passwords"]
struct NewPasswordEntry<'a> {
    password: &'a str,
    notes: &'a str,
    created_at: String,
}

impl Password {
    pub fn is_authorized(
        db: &SqliteConnection,
        provided_password: &MacRomanString<255>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        use crate::schema::passwords::dsl::*;

        let provided_password = provided_password.as_string();

        let results = passwords
            .filter(password.eq(provided_password))
            .limit(1)
            .load::<Password>(db)?;

        Ok(results.len() == 1)
    }

    pub fn add(
        db: &SqliteConnection,
        password: &str,
        notes: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let new_password = NewPasswordEntry {
            password,
            notes,
            created_at: now(),
        };

        diesel::insert_into(passwords::table)
            .values(&new_password)
            .execute(db)?;

        Ok(())
    }

    pub fn remove(
        db: &SqliteConnection,
        password_to_delete: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::schema::passwords::dsl::*;

        diesel::delete(passwords.filter(password.eq(password_to_delete))).execute(db)?;

        Ok(())
    }

    pub fn list(db: &SqliteConnection) -> Result<Vec<Password>, Box<dyn std::error::Error>> {
        use crate::schema::passwords::dsl::*;

        let results = passwords.load::<Password>(db)?;

        Ok(results)
    }

    pub fn len(db: &SqliteConnection) -> Result<i64, Box<dyn std::error::Error>> {
        use crate::schema::passwords::dsl::*;

        // FIXME: surely there's a better way to do this.
        let result = passwords.filter(id.gt(0)).count().first(db)?;

        Ok(result)
    }
}
