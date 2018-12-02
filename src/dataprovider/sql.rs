use crate::ratings::{RatingContainer, RatingContainerBuilder};
use super::RatingDataProvider;

use mysql as my;

pub struct SQLDataProvider {
    connection_string: String,
    query: String
}
impl SQLDataProvider {
    pub fn new(connection_string: &str, aid_name: &str, uid_name: &str, rating_name: &str, table_name: &str) -> Self {
        return Self {
            connection_string: connection_string.to_owned(),
            query: format!("SELECT {}, {}, {} FROM {}", aid_name, uid_name, rating_name, table_name)
        };
    }
}

impl RatingDataProvider for SQLDataProvider {
    fn get(&self) -> RatingContainer {
        let mut rating_builder = RatingContainerBuilder::new();

        let pool = my::Pool::new(&self.connection_string).unwrap();
        match pool.prep_exec(&self.query, ()) {
            Ok(result) => {
                result.map(|row| row.unwrap()).for_each(|row| {
                    let (animeid, userid, rating) = my::from_row(row);
                    rating_builder.add_rating(animeid, userid, rating)
                });
            },
            Err(e) => { panic!(e); }
        }

        return rating_builder.build();
    }
}