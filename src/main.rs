use html_parser::{Dom, Element, Node};
use std::collections::hash_map::HashMap;
mod food_item;

use food_item::FoodItem;
use std::fs::write;
use std::fs::read_to_string;
use clap::Parser;
use std::path::PathBuf;

static BASE_URL: &str = "https://starbounder.org";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args{

    #[arg(short, long)]
    datafile: PathBuf,
    #[arg(short, long, default_value_t = false)]
    update: bool,
    #[arg(short, long, default_value_t = false)]
    analyze: bool,
    #[arg(short, long)]
    resultfile: PathBuf

}

fn main() -> Result<(), &'static str>{
    let args = Args::parse();
    if !args.update && !args.analyze{
        return Err("not instructed to do anything, you must either update or analyze")
    }
    let data = match args.update{
        true => update_data(args.datafile),
        false => {
            let raw = read_to_string(args.datafile).map_err(|_| "could find the file")?;
            serde_json::from_str(&raw).map_err(|_| "could not parse the file")?
        },
    };
    if args.analyze{
        let mut res = data.iter().map(|f| (f.name.clone(), f.net_value())).collect::<Vec<(String, usize)>>();
        res.sort_by_key(|e| e.1);
        res.reverse();
        write(args.resultfile, serde_json::to_string_pretty(&res).map_err(|_| "Failed to serialize results")?).map_err(|_| "failed to write file")?;
    }
    Ok(())
}

fn update_data(p: PathBuf) -> Vec<FoodItem>{
    let mut fil: Vec<FoodItem> = vec![];
    let mut cache: HashMap<String, FoodItem> = HashMap::new();
    let fl = get_food_href();
    let l = fl.len();
    for (n, (name, href)) in fl.iter().enumerate() {
        println!("Processing item {n}/{l}");
        let item = match cache.get(name) {
            Some(i) => i.clone(),
            None => {
                let (fi, c) = get_item_info(name, href, cache).unwrap();
                cache = c;
                fi
            }
        };
        fil.push(item);
    }
    write(p, serde_json::to_string(&fil).unwrap()).unwrap();
    fil
}

fn get_food_href() -> Vec<(String, String)> {
    get_food_list()
        .unwrap()
        .iter()
        .map(|element| -> (String, String) {
            let name = element.attributes.get("title").unwrap().as_ref().unwrap();
            let href = element.attributes.get("href").unwrap().as_ref().unwrap();
            (name.to_string(), href.to_string())
        })
        .collect()
}

fn get_food_list() -> Option<Vec<Element>> {
    let b = reqwest::blocking::get("https://starbounder.org/Food")
        .unwrap()
        .text()
        .unwrap();
    let dom = Dom::parse(&b).unwrap();
    if let Some(i) = dom.children.first() {
        return parse_childs_until(i.element().unwrap().clone(), |e| {
            e.id == Some("navbox".to_owned())
        })
        .map(|e| {
            return_matches(
                Node::Element(e),
                |e1| match e1.element() {
                    Some(e2) => e2.classes.contains(&"navboxlist".to_string()),
                    None => false,
                },
                vec![],
            )
            .iter()
            .flat_map(|e1| e1.element().unwrap().children.to_owned())
            .filter(|e1| e1.element().is_some())
            .map(|e1| e1.element().unwrap().to_owned())
            .collect()
        });
    }
    None
}

fn return_matches(d: Node, predicate: fn(&Node) -> bool, original: Vec<Node>) -> Vec<Node> {
    let mut v = original;
    if predicate(&d) {
        v.push(d);
        return v;
    }
    match d.element() {
        Some(e) => {
            for c in e.children.clone() {
                v = return_matches(c, predicate, v.clone());
            }
            v
        }
        None => v,
    }
}

fn parse_childs_until(d: Element, predicate: fn(&Element) -> bool) -> Option<Element> {
    if predicate(&d) {
        Some(d)
    } else {
        for e in d.children {
            if let Some(el) = e.element() {
                if let Some(e) = parse_childs_until(el.clone(), predicate) {
                    return Some(e);
                }
            }
        }
        None
    }
}

fn get_item_info(
    name: &str,
    url: &str,
    mut cache: HashMap<String, FoodItem>,
) -> Option<(FoodItem, HashMap<String, FoodItem>)> {
    match cache.get(url) {
        Some(f) => Some((f.clone(), cache)),
        None => {
            let b = reqwest::blocking::get(BASE_URL.to_string() + url)
                .ok()?
                .text()
                .ok()?;
            let dom = Dom::parse(&b).ok()?;
            let elem = dom.children.first()?.element()?.to_owned();
            let v = parse_childs_until(elem.clone(), |e| {
                e.classes.contains(&"vector_info_footer".to_string())
            })?;
            let (ingredients, c) = match parse_childs_until(elem, |e| {
                e.attributes
                    .get("style")
                    .map(|s| s.clone() == Some("width:300px;".to_string()))
                    .unwrap_or(false)
                    && e.children
                        .iter()
                        .find(|&ec1| {
                            ec1.element()
                                .map(|ec2| ec2.classes.iter().any(|ec3| ec3 == "crafting_top_bg"))
                                .unwrap_or(false)
                        })
                        .unwrap()
                        .element()
                        .unwrap()
                        .children
                        .iter()
                        .any(|ec2| {
                            ec2.element()
                                .unwrap()
                                .children
                                .iter()
                                .any(|ec3| ec3.text() == Some("INGREDIENTS"))
                        })
            }) {
                Some(i) => parse_ingredients(i, cache.clone()),
                None => (vec![], cache.clone()),
            };
            cache = c;
            let p = find_price(v)?;
            let fi = FoodItem {
                name: name.to_string(),
                price: p,
                ingredients,
            };
            cache.insert(name.to_string(), fi.clone());
            Some((fi, cache))
        }
    }
}

fn find_price(e: Element) -> Option<usize> {
    for c in e.children.iter().map(|e| e.element()) {
        let t: String = c?.children.iter().flat_map(|e| e.text()).collect();
        if let Ok(u) = t.parse::<usize>() {
            return Some(u);
        }
    }
    None
}

fn parse_ingredients(
    element: Element,
    mut cache: HashMap<String, FoodItem>,
) -> (Vec<(usize, FoodItem)>, HashMap<String, FoodItem>) {
    let csb: Vec<Element> = element
        .children
        .iter()
        .filter_map(|e| e.element())
        .map(|e| e.to_owned())
        .filter(|e| e.classes.contains(&"crafting_stat_bg".to_string()))
        .collect();
    let pil: Vec<(usize, String, String)> = csb.iter().filter_map(|e| {
        e.children.iter().find(|e1| {
            e1.element().unwrap().attributes.get("style").map(|e2| {
                e2.clone()
                    == Some("text-align:left; padding-top:6px; margin-left:9px;".to_string())
            }).unwrap_or(false)
        })
    }).filter_map(|e1| e1.element()).map(|e1| {
        let mut name = None;
        let mut href = None;
        let mut amount = None;
        for c in e1.children.iter().map(|e2| e2.element().unwrap()){
            if let Some(n) = c.attributes.get("title").map(|a| a.clone().unwrap()){
                name = Some(n)
            }
            if let Some(n) = c.attributes.get("href").map(|a| a.clone().unwrap()){
                href = Some(n)
            }
            if let Some(n) = c.attributes.get("style"){
                if *n == Some("float:right; color: white; padding-right: 8px;font-size:20px;padding-top:20px;".to_string()){
                    amount = c.children.iter().find_map(|n2| n2.text()).map(|s| s.parse::<usize>())
                }
            }
        }
        (amount.unwrap().unwrap(), name.unwrap(), href.unwrap())
    }).collect();
    let fil: Vec<(usize, FoodItem)> = pil
        .iter()
        .map(|item| match cache.get(&item.1) {
            Some(f) => (item.0, f.clone()),
            None => {
                let (fi, c) = get_item_info(&item.1, &item.2, cache.clone()).unwrap();
                cache = c;
                (item.0, fi)
            }
        })
        .collect();
    (fil, cache)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_item_info() {
        let n = "Cooked Ribs";
        let url = "/Cooked_Ribs";
        let (fi, c) = get_item_info(n, url, HashMap::new()).unwrap();
        assert_eq!(fi.name, n);
        assert_eq!(fi.price, 40);
        println!("{:#?}", c)
    }
    #[test]
    #[ignore = "just prints"]
    fn test_get_food_list() {
        let l = get_food_list().unwrap();
        println!("length is {:?}", l.len());
        println!("{:#?}", l.first().unwrap());
    }
    #[test]
    #[ignore = "just prints"]
    fn test_get_food_href() {
        let l = get_food_href();
        println!("{:#?}", l)
    }
}
