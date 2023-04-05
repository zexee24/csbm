#[derive(Clone, Debug)]
pub struct FoodItem {
    pub name: String,
    pub price: usize,
    pub ingredients: Vec<(usize, FoodItem)>,
}
