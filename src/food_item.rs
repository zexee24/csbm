use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FoodItem {
    pub name: String,
    pub price: usize,
    pub ingredients: Vec<(usize, FoodItem)>,
}

impl FoodItem {
    pub fn get_ingredient_value(&self) -> usize {
        match self.ingredients.is_empty() {
            true => self.price,
            false => self.ingredients.iter().fold(0, |s, x| s + x.1.price * x.0),
        }
    }
    pub fn net_value(&self) -> usize {
        self.price - self.get_ingredient_value()
    }
    pub fn canmake(&self, vs: Vec<String>) -> bool {
        for v in vs {
            if !self
                .ingredients
                .iter()
                .map(|(_, n)| n.name.to_owned())
                .collect::<Vec<String>>()
                .contains(&v)
            {
                return false;
            }
        }
        true
    }
    pub fn canmaker(&self, vs: &Vec<String>) -> bool {
        if vs.contains(&self.name) {
            true
        } else {
            if self.ingredients.is_empty() {
                return false;
            }
            self.ingredients.iter().fold(true, |mut acc, (_, e)| {
                if acc {
                    let a = e.canmaker(vs);
                    acc = a;
                    acc
                } else {
                    false
                }
            })
        }
    }
    pub fn get_eff(&self) -> f64 {
        (self.price as f64) / (self.get_ingredient_value() as f64)
    }
}
