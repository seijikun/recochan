extern crate csv;
extern crate mysql;

mod testdata;
mod sql;
mod unittestdata;
use crate::ratings::RatingContainer;

pub use self::testdata::TestDataCsvProvider;
pub use self::sql::SQLDataProvider;
pub use self::unittestdata::UnitTestDataProvider;

pub trait RatingDataProvider {
    fn get(&self) -> RatingContainer;
}
