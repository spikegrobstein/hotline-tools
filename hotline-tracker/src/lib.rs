pub mod header;
mod registration_record;
mod server_record;
mod update_record;

pub use header::Header;
pub use registration_record::RegistrationRecord;
pub use server_record::ServerRecord;
pub use update_record::UpdateRecord;

#[derive(Debug)]
pub enum TrackerPacket {
    Header,
    Update(UpdateRecord),
    Server(Box<ServerRecord>),
    Complete,
}
