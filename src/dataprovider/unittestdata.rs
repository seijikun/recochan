use crate::ratings::{Id, RatingValue, RatingContainer, RatingContainerBuilder};
use super::RatingDataProvider;

pub struct UnitTestDataProvider {
	data: Vec<(Id, Id, RatingValue)>
}

impl UnitTestDataProvider {
    pub fn new(data: Vec<(Id, Id, RatingValue)>) -> Self {
		UnitTestDataProvider{ data }
    }
}

impl RatingDataProvider for UnitTestDataProvider {
    fn get(&self) -> RatingContainer {
		let mut rating_builder = RatingContainerBuilder::new();

		for d in &self.data {
			rating_builder.add_rating(d.0, d.1, d.2);
		}

		rating_builder.build()
    }
}


