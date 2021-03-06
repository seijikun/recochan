use nalgebra as na;
use kdtree::KdTree;
use serde_derive::Serialize;
use std::{fmt, sync::RwLock, iter::FromIterator};
use crate::ratings::{Id, RatingValue, RatingContainer};
use crate::dataprovider::RatingDataProvider;

#[derive(Debug)]
pub enum PredictionError {
    Unknown,
    NotInitialized,
    UnknownUser,
    UnknownAnime
}
impl fmt::Display for PredictionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PredictionError::Unknown => write!(f, "An unknown error occured during prediction."),
            PredictionError::NotInitialized => write!(f, "RecommendationEngine needs to be initialized by calling retrain() first."),
            PredictionError::UnknownUser => write!(f, "Couldn't find a user with this id"),
            PredictionError::UnknownAnime => write!(f, "Couldn't find an anime with this id")
        }
    }
}


#[derive(Serialize)]
pub struct UserRatingPrediction{ pub animeid: Id, pub rating: RatingValue }
pub type UserRatingPredictionResult = Vec<UserRatingPrediction>;
#[derive(Serialize)]
pub struct SimilarUser{ pub userid: Id, pub similarity: RatingValue }
pub type SimilarUserResult = Vec<SimilarUser>;
#[derive(Serialize)]
pub struct SimilarAnime{ pub animeid: Id, pub similarity: RatingValue }
pub type SimilarAnimeResult = Vec<SimilarAnime>;


pub type PredictionSanitizerFn = dyn Fn(RatingValue) -> RatingValue + Send + Sync;
pub static PREDICTION_SANITIZER_NOOP: &PredictionSanitizerFn = &|rating_value: RatingValue| {
    return rating_value;
};
pub static PREDICTION_SANITIZER_CLAMP: &PredictionSanitizerFn = &|rating_value: RatingValue| {
    if rating_value < 0.0 { return 0.0; }
    if rating_value > 5.0 { return 5.0; }
    return rating_value;
};


/// Configuration structure that contains a couple of parameters
/// internally used by the RecommendationEngine. The default values
/// are the ones providing the best result when using the test data.
pub struct RecommendationEngineConf {
    features: usize,
    learn_rate: RatingValue,
    min_steps: usize,
    max_steps: usize,
    min_improvement: RatingValue,
    regularization_parameter: RatingValue,
    prediction_sanitizer: &'static PredictionSanitizerFn,
    initial_approximation_value: RatingValue,
    k: RatingValue
}
impl Default for RecommendationEngineConf {
    fn default() -> Self {
        return Self {
            features: 25,
            learn_rate: 0.01,
            min_steps: 25,
            max_steps: 120,
            min_improvement: 0.00001,
            regularization_parameter: 0.02,
            prediction_sanitizer: PREDICTION_SANITIZER_CLAMP,
            initial_approximation_value: 0.1,
            k: 25.0
        };
    }
}


/// Struct that contains the RecommendationEngine's internal
/// (thread-safe) state. This contains the data that is used to
/// make recommendations. After the engine is done re-training
/// the state will be locked and swapped with the new one.
struct RecommendationEngineState {
    // General rating statistics
    global_rating_avg: RatingValue,
    global_avg_offset: RatingValue,

    // Anime-specific rating statistics
    anime_rating_cnt: na::DVector<usize>,
    anime_rating_avg: na::DVector<RatingValue>,

    // Anime-specific rating statistics
    user_rating_cnt: na::DVector<usize>,
    user_avg_offset: na::DVector<RatingValue>,

    // Feature matrices that are calculated using the funkSVD.
    // These are the matrices used to make a personalized prediction.
    anime_features: na::DMatrix<RatingValue>,
    user_features: na::DMatrix<RatingValue>,

    // Spatial indices of the feature-vectors within the feature matrices
    // above. These are used for finding similar user2user and anime2anime.
    anime_feature_tree: KdTree<RatingValue, Id, Vec<RatingValue>>,
    user_feature_tree: KdTree<RatingValue, Id, Vec<RatingValue>>,

    // model statistics
    ratings: RatingContainer,
    approximation_error: RatingValue
}
impl RecommendationEngineState {
    pub fn new(ratings: RatingContainer) -> Self {
        return Self {
            global_rating_avg: 0.0, global_avg_offset: 0.0,
            anime_rating_cnt: na::DVector::from_element(0,0), anime_rating_avg: na::DVector::from_element(0,0.0),
            user_rating_cnt: na::DVector::from_element(0,0), user_avg_offset: na::DVector::from_element(0,0.0),
            anime_features: na::DMatrix::from_element(0,0,0.0), user_features: na::DMatrix::from_element(0,0,0.0),
            anime_feature_tree: KdTree::new(0), user_feature_tree: KdTree::new(0),
            approximation_error: 0.0, ratings
        };
    }
}


/// This is beautiful RecoChan. It's a latent factor analysis based
/// recommendation engine. This is essentially the method that made
/// 3rd place in the Netflix-Challenge in 2006.
pub struct RecommendationEngine {
    config: RecommendationEngineConf,

    rating_provider: Box<dyn RatingDataProvider + Send + Sync>,
    state: RwLock<Option<RecommendationEngineState>>
}
impl RecommendationEngine {
    pub fn new(config: RecommendationEngineConf, rating_provider: Box<dyn RatingDataProvider + Send + Sync>) -> Self {
        return Self {
            config, rating_provider: rating_provider,
            state: RwLock::new(None)
        };
    }

    pub fn new_default(rating_provider: Box<dyn RatingDataProvider + Send + Sync>) -> Self {
        return Self::new(RecommendationEngineConf::default(), rating_provider);
    }

    /// This method will acquire the current list of ratings from the configured
    /// data-source, use that to train a new model and then swap the current model with the
    /// newly trained one.
    pub fn retrain(&self) {
        info!(target: "RecommendationEngine", "Start training...");
        let rating_data = self.rating_provider.get();
        let mut state = RecommendationEngineState::new(rating_data);
        self.init_statistics(&mut state);
        let ratings = &state.ratings.ratings;
        let conf = &self.config;
        let (anime_rating_avg, user_avg_offset) = (&state.anime_rating_avg, &state.user_avg_offset);

        state.anime_features = na::DMatrix::from_element(anime_rating_avg.len(), conf.features,
                                                        conf.initial_approximation_value);
        state.user_features = na::DMatrix::from_element(conf.features, user_avg_offset.len(),
                                                        conf.initial_approximation_value);
        let mut residual_cache: Vec<_> = ratings.iter()
                .map(|r| anime_rating_avg[r.animeidx] + user_avg_offset[r.useridx]).collect();

        for f in 0..conf.features {
            let mut improvement = std::f32::MAX as RatingValue; // Just use something reasonably big here

            state.approximation_error = Self::evaluate_model(&state, &residual_cache, f);
            debug!(target: "RecommendationEngine", "Training feature {}...", f);
            debug!(target: "RecommendationEngine", "Approximation-Error: {}", state.approximation_error);

            let mut i = 0;
            while i < conf.min_steps || (i < conf.max_steps && improvement > conf.min_improvement) {
                for (idx, rating) in ratings.iter().enumerate() {
                    let (a,u) = (rating.animeidx, rating.useridx);
                    
                    let prediction = residual_cache[idx] + state.anime_features[(a,f)] * state.user_features[(f,u)];
                    let err = rating.rating - prediction;

                    let af = state.anime_features[(a,f)];
                    let uf = state.user_features[(f,u)];
                    state.anime_features[(a,f)] += conf.learn_rate * (err * uf - conf.regularization_parameter * af);
                    state.user_features[(f,u)] += conf.learn_rate * (err * af - conf.regularization_parameter * uf);
                }

                let error = Self::evaluate_model(&state, &residual_cache, f);
                improvement = state.approximation_error - error;
                state.approximation_error = error;
                debug!(target: "RecommendationEngine", "Approximation-Error: {}", state.approximation_error);

                i += 1;
            }

            // Apply trained model to cache and continue with next round
            for (idx, rating) in ratings.iter().enumerate() {
                let (a,u) = (rating.animeidx, rating.useridx);
                residual_cache[idx] = (conf.prediction_sanitizer)(
                    residual_cache[idx] + state.anime_features[(a,f)] * state.user_features[(f,u)]
                );
            }
        }

        // Build KdTrees for anime/user features for a fast knn neighbor search.
        info!(target: "RecommendationEngine", "Building spatial indices for anime- / user-features");
        state.anime_feature_tree = KdTree::new_with_capacity(conf.features, state.anime_features.nrows());
        for a in 0..state.anime_features.nrows() {
            let point_vec = Vec::from_iter(state.anime_features.row(a).iter().map(|r| *r));
            let animeid = state.ratings.row2anime(a).unwrap();
            state.anime_feature_tree.add(point_vec, animeid).expect("Building spatial tree for anime features failed");
        }
        state.user_feature_tree = KdTree::new_with_capacity(conf.features, state.user_features.ncols());
        for u in 0..state.user_features.ncols() {
            let point_vec = Vec::from_iter(state.user_features.column(u).iter().map(|r| *r));
            let userid = state.ratings.column2user(u).unwrap();
            state.user_feature_tree.add(point_vec, userid).expect("Building spatial tree for user features failed");
        }

        // Swap newly trained state with the state that is currently used for predictions
        // Unwrap is ok here. Can only fail if a writer panics - which will not happen.
        info!(target: "RecommendationEngine", "Finished training - swapping with active EngineState");
        info!(target: "RecommendationEngine", "Average Prediction Error: {:.3} Stars", state.approximation_error);
        *self.state.write().unwrap() = Some(state);
    }

    fn evaluate_model(state: &RecommendationEngineState, residual_cache: &Vec<RatingValue>, f: usize) -> RatingValue {
        let mut result = 0.0 as RatingValue;
        for (idx,rating) in state.ratings.ratings.iter().enumerate() {
            let (a,u) = (rating.animeidx, rating.useridx);
            let predicition = residual_cache[idx] + state.anime_features[(a,f)] * state.user_features[(f,u)];
            result += f64::abs(rating.rating - predicition);
        }
        return result / state.ratings.ratings.len() as RatingValue;
    }

    fn init_statistics(&self, state: &mut RecommendationEngineState) {
        let (ratings, animes, users) = (&state.ratings.ratings, &state.ratings.animes, &state.ratings.users);
        let (global_rating_avg, global_avg_offset, k) = (&mut state.global_rating_avg, &mut state.global_avg_offset, self.config.k);
        let (anime_rating_cnt, anime_rating_avg) = (&mut state.anime_rating_cnt, &mut state.anime_rating_avg);
        let (user_rating_cnt, user_avg_offset) = (&mut state.user_rating_cnt, &mut state.user_avg_offset);

        *anime_rating_cnt = na::DVector::from_element(animes.len(), 0);
        *anime_rating_avg = na::DVector::from_element(animes.len(), 0.0);
        *user_rating_cnt = na::DVector::from_element(users.len(), 0);
        *user_avg_offset = na::DVector::from_element(users.len(), 0.0);

        // Calculate global rating average
        // Calculate average rating per anime
        for rating in ratings {
            anime_rating_cnt[rating.animeidx] += 1;
            anime_rating_avg[rating.animeidx] += rating.rating;
            *global_rating_avg += rating.rating;
        }
        *global_rating_avg /= ratings.len() as RatingValue;
        for idx in 0..animes.len() {
            anime_rating_avg[idx] = (*global_rating_avg * k + anime_rating_avg[idx]) / (k + anime_rating_cnt[idx] as RatingValue);
        }

        // Calculate average user rating-offset
        for rating in ratings {
            user_rating_cnt[rating.useridx] += 1;
            user_avg_offset[rating.useridx] += rating.rating - anime_rating_avg[rating.animeidx];
            *global_avg_offset += rating.rating - anime_rating_avg[rating.animeidx];
        }
        *global_avg_offset /= ratings.len() as RatingValue;
        for idx in 0..users.len() {
            user_avg_offset[idx] = (*global_avg_offset * k + user_avg_offset[idx]) / (k + user_rating_cnt[idx] as RatingValue);
        }
    }

    fn use_state<F,T>(&self, cb: F) -> Result<T, PredictionError>
                where F: FnOnce(&RecommendationEngineState) -> Result<T, PredictionError> {
        let state_lock = self.state.read().map_err(|_| PredictionError::Unknown)?;
        let state = state_lock.as_ref().ok_or(PredictionError::NotInitialized)?;
        return cb(state);
    }

    pub fn predict_user_ratings<P>(&self, userid: Id, filter: P) -> Result<UserRatingPredictionResult, PredictionError> 
                where P: FnMut(&UserRatingPrediction) -> bool {
        return self.use_state(|state| {
            let useridx = state.ratings.user2column(userid).ok_or(PredictionError::UnknownUser)?;

            // Calculate predictions for every known anime for the given user
            let predictions = &state.anime_features * state.user_features.column(useridx)
                                    + &state.anime_rating_avg.add_scalar(state.user_avg_offset[useridx]);

            let mut result: UserRatingPredictionResult = state.ratings.animes.iter().enumerate()
                                                .map(|(idx, a)| UserRatingPrediction {
                                                    animeid: a.id,
                                                    rating: (self.config.prediction_sanitizer)(predictions[idx])
                                                })
                                                .filter(filter)
                                                .collect();
            // Sort predicated ratings (ascending)
            result.sort_by(|p0, p1| p1.rating.partial_cmp(&p0.rating).unwrap_or(std::cmp::Ordering::Greater));

            return Ok(result);
        });
    }

    pub fn find_k_similar_users(&self, userid: Id, k: usize) -> Result<SimilarUserResult, PredictionError> {
        return self.use_state(|state| {
            let useridx = state.ratings.user2column(userid).ok_or(PredictionError::UnknownUser)?;
            
            // Get a many-dimensional point representing the user in question within the user-feature-space.
            let user_point = Vec::from_iter(state.user_features.column(useridx).iter().map(|r| *r));
            
            // Use the spatial tree to search for users near ours in user-feature-space.
            // We request one more because the search will also return our anime itself
            let similar_users = state.user_feature_tree.nearest(&user_point, k + 1, &|u0, u1| {
                u0.iter().zip(u1).map(|(u0v, u1v)| (u0v - u1v).powf(2.0)).sum::<RatingValue>()
            }).map_err(|_| PredictionError::Unknown)?;

            return Ok(similar_users.into_iter().filter_map(|(distance, &other_uid)|
                if userid != other_uid {
                    Some(SimilarUser{ userid: other_uid, similarity: 1.0 / distance })
                } else { None }
            ).collect());
        });
    }

    pub fn find_k_similar_animes(&self, animeid: Id, k: usize) -> Result<SimilarAnimeResult, PredictionError> {
        return self.use_state(|state| {
            let animeidx = state.ratings.anime2row(animeid).ok_or(PredictionError::UnknownAnime)?;
            
            // Get a many-dimensional point representing the anime in question within the anime-feature-space.
            let anime_point = Vec::from_iter(state.anime_features.row(animeidx).iter().map(|r| *r));
            
            // Use the spatial tree to search for users near ours in anime-feature-space.
            // We request one more because the search will also return our anime itself
            let similar_animes = state.anime_feature_tree.nearest(&anime_point, k + 1, &|a0, a1| {
                a0.iter().zip(a1).map(|(a0v, a1v)| (a0v - a1v).powf(2.0)).sum::<RatingValue>()
            }).map_err(|_| PredictionError::Unknown)?;

            return Ok(similar_animes.into_iter().filter_map(|(distance, &other_aid)|
                if animeid != other_aid {
                    Some(SimilarAnime{ animeid: other_aid, similarity: 1.0 / distance })
                } else { None }
            ).collect());
        });
    }
}