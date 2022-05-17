use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::default::Default;

use tokio::time::{Instant, Duration};

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
    server_expiry: Duration,
    servers: HashMap<u32, ServerEntry>,
}

impl Default for ServerRegistry {
    fn default() -> Self {
        Self {
            server_expiry: Duration::from_secs(300), // 5 minutes
            servers: HashMap::new(),
        }
    }
}

impl ServerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expire(&mut self) {
        self.servers.retain(|&k, v| {
            let expires_at = v.datestamp + self.server_expiry;

            eprintln!("server record expires_at: {:?}", expires_at);

            // duration == 0 //-> not expired
            let is_expired = Instant::now().duration_since(expires_at) != Duration::ZERO;

            if is_expired {
                eprintln!("server {k} expired.");
            }

            // return true to keep
            !is_expired
        });
    }

    pub fn register(&mut self, address: Ipv4Addr, registration_record: RegistrationRecord) {
        let id = registration_record.id;
        let server = registration_record.to_server_record(address);

        self.servers.insert(id, ServerEntry::new(server));
    }

    pub fn create_update_record(&mut self) -> UpdateRecord {
        self.expire();

        let remaining_data_size: u16 = self.servers
            .iter()
            .map(|(_, entry)| entry.server.data_size() as u16)
            .sum();

        let total_servers = self.servers.len() as u16;

        // TODO: don't hard-code this version number
        UpdateRecord {
            version: 1,
            remaining_data_size,
            total_servers,
            remaining_servers: total_servers,
        }
    }

    pub fn server_records(&mut self) -> Vec<ServerRecord> {
        self.servers.iter().map(|(_k, v)| {
            v.server.clone()
        })
        .collect()
    }
}
