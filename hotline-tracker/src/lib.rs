mod update_record;
mod server_record;

pub use update_record::UpdateRecord;
pub use server_record::ServerRecord;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
