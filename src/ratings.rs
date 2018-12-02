use std::collections::{HashSet, HashMap};

pub type Id = u64;
pub type Index = usize;
pub type RatingValue = f64;

pub struct Anime {
    pub id: Id
}
impl Anime {
    pub fn new(id: Id) -> Self {
        return Self {id};
    }
}

pub struct User {
    pub id: Id
}
impl User {
    pub fn new(id: Id) -> Self {
        return Self {id};
    }
}

pub struct Rating {
    pub animeidx: Index,
    pub useridx: Index,
    pub rating: RatingValue
}

pub struct RatingContainer {
    pub ratings: Vec<Rating>,
    pub animes: Vec<Anime>,
    pub users: Vec<User>,
    anime2row: HashMap<Id, Index>,
    row2anime: HashMap<Index, Id>,
    user2column: HashMap<Id, Index>,
    column2user: HashMap<Index, Id>
}
impl RatingContainer {
    pub fn anime2row(&self, animeid: Id) -> Option<Index> {
        return self.anime2row.get(&animeid).map(|r| *r);
    }
    pub fn row2anime(&self, rowidx: Index) -> Option<Id> {
        return self.row2anime.get(&rowidx).map(|a| *a);
    }
    pub fn user2column(&self, userid: Id) -> Option<Index> {
        return self.user2column.get(&userid).map(|c| *c);
    }
    pub fn column2user(&self, columnidx: Index) -> Option<Id> {
        return self.column2user.get(&columnidx).map(|u| *u);
    }
}



pub struct RatingContainerBuilder {
    ratings: Vec<(Id, Id, RatingValue)>,
    anime_ids: HashSet<Id>,
    user_ids: HashSet<Id>
}
impl RatingContainerBuilder {
    pub fn new() -> Self {
        return Self {
            ratings: Vec::new(), anime_ids: HashSet::new(), user_ids: HashSet::new()
        }
    }
    
    pub fn add_rating(&mut self, animeid: Id, userid: Id, rating: RatingValue) {
        self.anime_ids.insert(animeid);
        self.ratings.push((animeid, userid, rating));
        self.user_ids.insert(userid);
    }

    pub fn build(self) -> RatingContainer {
        // Generate sequential ids for animes
        let mut animeid_list: Vec<Id> = self.anime_ids.into_iter().collect();
        animeid_list.sort();
        // Generate (back&forth) maps for animes
        let anime2row: HashMap<Id, Index> = animeid_list.iter().enumerate()
                    .map(|(idx, &anime_id)| (anime_id, idx as Index) ).collect();
        let row2anime: HashMap<Index, Id> = animeid_list.iter().enumerate()
                    .map(|(idx, &anime_id)| (idx as Index, anime_id) ).collect();

        // Generate sequential ids for animes
        let mut userid_list: Vec<Id> = self.user_ids.into_iter().collect();
        userid_list.sort();
        // Generate (back&forth) maps for animes
        let user2column: HashMap<Id, Index> = userid_list.iter().enumerate()
                    .map(|(idx, &user_id)| (user_id, idx as Index) ).collect();
        let column2user: HashMap<Index, Id> = userid_list.iter().enumerate()
                    .map(|(idx, &user_id)| (idx as Index, user_id) ).collect();

        // Generate anime and user arrays
        let animes: Vec<Anime> = animeid_list.iter().map(|&animeid| Anime::new(animeid)).collect();
        let users: Vec<User> = userid_list.iter().map(|&userid| User::new(userid)).collect();

        let mut ratings: Vec<Rating> = self.ratings.into_iter().map(|(animeid, userid, rating)| {
            Rating { animeidx: anime2row[&animeid], useridx: user2column[&userid], rating: rating }
        }).collect();
        // Sort ratings by animeidx, then by useridx
        ratings.sort_unstable_by(|r0,r1| {
            if r0.animeidx.cmp(&r1.animeidx) != std::cmp::Ordering::Equal {
                r0.animeidx.cmp(&r1.animeidx)
            } else { r0.useridx.cmp(&r1.useridx) }
        });

        return RatingContainer {
            ratings, animes, users,
            anime2row, row2anime, user2column, column2user
        };
    }
}

