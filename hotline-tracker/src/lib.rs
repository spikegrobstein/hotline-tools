mod update_record;
mod server_record;
mod registration_record;
pub mod header;

pub use update_record::UpdateRecord;
pub use server_record::ServerRecord;
pub use registration_record::RegistrationRecord;
pub use header::Header;

#[derive(Debug)]
pub enum TrackerPacket {
    Header,
    Update(UpdateRecord),
    Server(Box<ServerRecord>),
    Complete,
}

