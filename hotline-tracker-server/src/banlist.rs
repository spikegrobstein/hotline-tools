use diesel::prelude::*;

use std::net::Ipv4Addr;

#[derive(Queryable)]
pub struct Banlist {
    pub id: i32,
    pub address: String,
    pub notes: String,
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
}
