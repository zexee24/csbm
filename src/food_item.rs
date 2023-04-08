use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FoodItem {
    pub name: String,
    pub price: usize,
    pub ingredients: Vec<(usize, FoodItem)>,
}
