extern crate csv;
extern crate mysql;

mod testdata;
use crate::ratings::RatingContainer;

pub use self::testdata::TestDataCsvProvider;

pub trait RatingDataProvider {
    fn get(&self) -> RatingContainer;
}
