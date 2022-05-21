use diesel::prelude::*;
use super::schema::banlist;

use std::net::Ipv4Addr;

#[derive(Queryable)]
pub struct Banlist {
    pub id: i32,
    pub address: String,
    pub notes: String,
}

#[derive(Insertable)]
#[table_name="banlist"]
struct NewBanlistEntry {
    address: String,
    notes: String,
}

impl Banlist {
    pub fn is_banned(db: &SqliteConnection, addr: &Ipv4Addr) -> Result<bool, Box<dyn std::error::Error>> {
        use crate::schema::banlist::dsl::*;

        let addr_str: String = format!("{addr}");

        let results = banlist.filter(address.eq(addr_str))
            .limit(1)
            .load::<Banlist>(db)?;

        Ok(results.len() == 1)
    }

    pub fn add(db: &SqliteConnection, address: String, notes: String) -> Result<(), Box<dyn std::error::Error>> {
        // todo: add a better validation error here
        let _: Ipv4Addr = address.parse()?;

        let new_banlist_entry = NewBanlistEntry {
            address,
            notes,
        };

        diesel::insert_into(banlist::table)
            .values(&new_banlist_entry)
            .execute(db)?;

        Ok(())
    }

    pub fn remove(db: &SqliteConnection, addr: String) -> Result<(), Box<dyn std::error::Error>> {
        use crate::schema::banlist::dsl::*;

        diesel::delete(banlist.filter(address.eq(addr)))
            .execute(db)?;

        Ok(())
    }

    pub fn list(db: &SqliteConnection) -> Result<Vec<Banlist>, Box<dyn std::error::Error>> {
        use crate::schema::banlist::dsl::*;

        let results = banlist
            .load::<Banlist>(db)?;

        Ok(results)
    }
}
