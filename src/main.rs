use html_parser::{Dom, Element, Node};
use std::collections::hash_map::HashMap;
mod food_item;

use food_item::FoodItem;

static BASE_URL: &str = "https://starbounder.org/";
fn main() {
    let fil: Vec<FoodItem>;
    let cache: HashMap<String, FoodItem> = HashMap::new();
    for child in get_food_list().unwrap(){
        let a = child.attributes;
        let name = a.get("title").unwrap().as_ref().unwrap();
        let href = a.get("href").unwrap().as_ref().unwrap();
        
    }
}

fn get_food_list() -> Option<Vec<Element>> {
    let b = reqwest::blocking::get("https://starbounder.org/Category:Food").unwrap().text().unwrap();
        let dom = Dom::parse(&b).unwrap();
        if let Some(i) = dom.children.first(){
            return match parse_childs_until(i.element().unwrap().clone(), |e| e.name == "ul".to_string()){
                Some(e) => Some(e.children.iter().map(|el| el.element().unwrap().to_owned().children.first().unwrap().element().unwrap().to_owned()).collect()),
                None => None,
            }
        }
        None
}

fn parse_childs_until(d: Element, predicate: fn(&Element) -> bool)-> Option<Element>{
    if predicate(&d) {Some(d)}
    else {
        for e in d.children{
            if let Some(el) = e.element(){
                if let Some(e) = parse_childs_until(el.clone(), predicate.clone()){
                    return Some(e)
                }
            }
        }
        None
    }
}

fn get_item_info (name: String, url: String,mut cache: HashMap<String, FoodItem>) -> Option<(FoodItem, HashMap<String, FoodItem>)>{
    match cache.get(&url){
        Some(f) => Some((f.clone(), cache)),
        None => {
            let b = reqwest::blocking::get(BASE_URL.to_string() + &url).ok()?.text().ok()?;
            let dom = Dom::parse(&b).ok()?;
            let v = parse_childs_until(dom.children.first()?.element()?.to_owned(), |e| e.classes.contains(&"vector_info_footer".to_string()))?;
            let p = find_price(v)?;
            let fi = FoodItem{ name: name.clone(), price: p, ingredients: vec![] };
            cache.insert(name, fi.clone());
            Some((fi, cache))
        }
    }
}

fn find_price(e: Element) -> Option<usize> {
    for c in e.children.iter().map(|e| e.element().unwrap()){
        let t : String = c.children.iter().flat_map(|e| e.text()).collect();
        if let Ok(u) = t.parse::<usize>() {return Some(u)}
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_item_info() {
        let n = "Cooked Ribs".to_string();
        let url = "Cooked_Ribs".to_string();
        let b = get_item_info(n.clone(),url,HashMap::new());
        println!("for item {:?} got {:?}",n,b);
    }
}
