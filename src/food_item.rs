use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FoodItem {
    pub name: String,
    pub price: usize,
    pub ingredients: Vec<(usize, FoodItem)>,
}

impl FoodItem{
    pub fn get_ingredient_value(&self) -> usize{
        match self.ingredients.is_empty(){
            true => 0,
            false => {
                self.ingredients.iter().fold(0, |s, x| s + x.1.price)
            },
        }
    }
    pub fn net_value(&self) -> usize{
        self.price - self.get_ingredient_value()
    }
}
