use crate::FoodItem;
#[derive(Clone, Debug)]
pub struct CraftInventory {
    items: Vec<(i32, FoodItem)>,
}

impl CraftInventory {
    pub fn new(items: Vec<(i32, FoodItem)>) -> Self {
        CraftInventory { items }
    }
    pub fn get_value(&self) -> usize {
        Self::calc_value(&self.items)
    }
    fn calc_value(items: &[(i32, FoodItem)]) -> usize {
        items
            .iter()
            .fold(0, |acc, f| acc + f.1.price * (f.0 as usize))
    }
    pub fn try_craft(&self, fi: &FoodItem) -> Result<Self, &Self> {
        let new = self.craft(fi);
        match new.is_viable() {
            true => Ok(new),
            false => Err(self),
        }
    }
    fn add_item(&mut self, fi: FoodItem, amount: i32) {
        match self.items.iter().find(|(_, f)| f == &fi) {
            Some((_, f)) => {
                self.items = self
                    .items
                    .iter()
                    .map(|(n2, f2)| {
                        if *f2 == fi {
                            (n2 + amount, f2.to_owned())
                        } else {
                            (*n2, f2.to_owned())
                        }
                    })
                    .collect()
            }
            None => self.items.push((amount, fi)),
        }
    }

    fn craft(&self, fi: &FoodItem) -> Self {
        let mut new = self.clone();
        for (n, ing) in fi.ingredients.clone() {
            if !self.items.iter().any(|(_, i)| *i == ing) {
                new = new.craft(&ing);
            }
            new.items = new
                .items
                .iter()
                .map(|i| {
                    if ing == i.1 {
                        (i.0 - (n as i32), i.1.to_owned())
                    } else {
                        i.to_owned()
                    }
                })
                .collect();
        }
        new.add_item(fi.clone(), 1);
        //println!("new is {:?}", new);
        new
    }
    fn is_viable(&self) -> bool {
        self.items.iter().fold(true, |acc, i| i.0 >= 0 && acc)
    }
    pub fn test_order(&self, recipes: &Vec<FoodItem>) -> usize{
        let mut ci = self.clone();
        for recipe in recipes {
            while let Ok(r) = ci.try_craft(recipe) {
                ci = r;
            }
        }
        ci.get_value()
    }
}
