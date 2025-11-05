pub mod model;
pub mod model_db;
pub mod db_context;
pub mod label_db;
pub mod audit_db;
pub mod blobs_db;
pub mod group_db;
pub mod label_keyword_db;
pub mod resource_db;
mod paginated_response;
pub use paginated_response::PaginatedResponse;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
