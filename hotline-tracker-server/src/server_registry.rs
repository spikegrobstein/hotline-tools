use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::default::Default;

use tokio::time::Instant;

use hotline_tracker::{
    ServerRecord,
    RegistrationRecord,
    UpdateRecord
};

#[derive(Debug)]
pub struct ServerEntry {
    datestamp: Instant,
    server: ServerRecord,
}

impl ServerEntry {
    pub fn new(server: ServerRecord) -> Self {
        Self {
            datestamp: tokio::time::Instant::now(),
            server,
        }
    }
}


/// servers that connect are listed here
/// contains a list of servers that have registered, when they last registered
/// and some other data about them.
#[derive(Debug)]
pub struct ServerRegistry {
    servers: HashMap<u32, ServerEntry>,
}

impl Default for ServerRegistry {
    fn default() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }
}

impl ServerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, address: Ipv4Addr, registration_record: RegistrationRecord) {
        let id = registration_record.id;
        let server = registration_record.to_server_record(address);

        self.servers.insert(id, ServerEntry::new(server));
    }

    pub fn create_update_record(&self) -> UpdateRecord {
        let users_online = self.servers
            .iter()
            .map(|(_, entry)| entry.server.users_online)
            .sum();

        let total_servers = self.servers.len() as u16;

        UpdateRecord {
            version: 1,
            users_online,
            total_servers,
            unknown: total_servers,
        }
    }
}
