use crate::ratings::{Id, RatingValue, RatingContainer, RatingContainerBuilder};
use super::RatingDataProvider;
use csv;
use std::path::Path;
use std::{io, fs};

pub struct TestDataCsvProvider {
    path: String
}

impl TestDataCsvProvider {
    pub fn new(path: &str) -> Self {
        return Self {
            path: path.to_owned()
        };
    }

    fn parse_file(path: &Path, animeid: Id, ratings: &mut RatingContainerBuilder) -> Result<(), io::Error> {
        let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(fs::File::open(path)?);
        for rating in rdr.records() {
            let rating = rating?;
            let userid = rating.get(0).unwrap().parse::<Id>().unwrap();
            // Move rating value into the range 0 to 5
            let rating = rating.get(1).unwrap().parse::<RatingValue>().unwrap() / 2.0;
            ratings.add_rating(animeid, userid, rating)
        }
        return Ok(());
    }
}

impl RatingDataProvider for TestDataCsvProvider {
    fn get(&self) -> RatingContainer {
        let mut rating_builder = RatingContainerBuilder::new();

        for file in fs::read_dir(&self.path).unwrap() {
            if let Ok(file) = file {
                let file_path = file.path();
                let extension = file_path.extension().unwrap().to_str().unwrap_or("");
                if file_path.is_file() && extension == "csv" {
                    let anime_id = file_path.file_stem().unwrap().to_str().unwrap_or("").parse::<Id>().unwrap();
                    let _ = Self::parse_file(&file.path(), anime_id, &mut rating_builder);
                }
            }
        }

        return rating_builder.build();
    }
}