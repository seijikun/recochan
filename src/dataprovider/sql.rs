use crate::ratings::{RatingContainer, RatingContainerBuilder, RatingValue};
use super::RatingDataProvider;

use mysql as my;

pub struct SQLDataProvider {
    connection_string: String,
    query: String
}
impl SQLDataProvider {
    pub fn new(connection_string: &str, where_clause: &str, aid_name: &str, uid_name: &str, rating_name: &str, table_name: &str) -> Self {
        let where_clause_str = match where_clause.len() {
            0 => "".to_owned(),
            _ => format!("WHERE {}", where_clause)
        };
        return Self {
            connection_string: connection_string.to_owned(),
            query: format!("SELECT {}, {}, {} FROM {} {}", aid_name, uid_name, rating_name, table_name, where_clause_str)
        };
    }
}

impl RatingDataProvider for SQLDataProvider {
    fn get(&self) -> RatingContainer {
        let mut rating_builder = RatingContainerBuilder::new();

        let pool = match my::Pool::new(&self.connection_string) {
            Ok(pool) => pool,
            Err(err) => {
                error!(target: "SQLDataProvider", "Error while trying to connect:\n{:?}", err);
                panic!();
            }
        };
        
        match pool.prep_exec(&self.query, ()) {
            Ok(result) => {
                result.map(|row| row.unwrap()).for_each(|row| {
                    let (animeid, userid, rating) : (u64, u64, u64) = my::from_row(row);
                    rating_builder.add_rating(animeid, userid, rating as RatingValue)
                });
            },
            Err(err) => {
                error!(target: "SQLDataProvider", "Error while trying fetch data:\n{:?}", err);
                panic!();
            }
        }

        return rating_builder.build();
    }
}