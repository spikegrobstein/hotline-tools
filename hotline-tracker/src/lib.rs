mod update_record;
mod server_record;
mod registration_record;
mod header;

pub use update_record::UpdateRecord;
pub use server_record::ServerRecord;
pub use registration_record::RegistrationRecord;
pub use header::Header;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
