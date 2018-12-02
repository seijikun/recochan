extern crate csv;
extern crate mysql;

mod testdata;
mod sql;
use crate::ratings::RatingContainer;

pub use self::testdata::TestDataCsvProvider;
pub use self::sql::SQLDataProvider;

pub trait RatingDataProvider {
    fn get(&self) -> RatingContainer;
}
