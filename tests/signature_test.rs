use furas::signature::*;
use scraper::{Html, Selector};


const SHOP_PAGE: &str = include_str!("./assets/shop.html");

#[test]
fn signature_generation() {
    let html = Html::parse_document(SHOP_PAGE);

    let selector = Selector::parse("div.col:nth-child(1)").unwrap();
    let el = html.select(&selector).next().unwrap();
    let signature = compute_signature(el);

    // println!("{:#?}", signature);

    assert_eq!(signature.tag_name, "div");
    assert_eq!(signature.depth, 5);
    assert_eq!(signature.index_in_parent, 0);
    assert_eq!(signature.has_children, true);

    let next_selector = Selector::parse("div.col:nth-child(2)").unwrap();
    let next_el = html.select(&next_selector).next().unwrap();

    // println!("next element {:?} ", next_el.value());

    let next_signature = compute_signature(next_el);
    
    assert_eq!(next_signature.tag_name, "div");
    assert_eq!(next_signature.depth, 5);
    assert_eq!(next_signature.index_in_parent, 1);
    assert_eq!(next_signature.has_children, true);
}

#[test]
fn signature_similarity() {
    let html = Html::parse_document(SHOP_PAGE);

    let selector = Selector::parse("div.col:nth-child(1)").unwrap();
    let el = html.select(&selector).next().unwrap();
    let mut signature = compute_signature(el);

    let next_selector = Selector::parse("div.col:nth-child(2)").unwrap();
    let next_el = html.select(&next_selector).next().unwrap();

    let score = compare_signature(&mut signature, el);
    let next_score = compare_signature(&mut signature, next_el);
    print!("score: {}, next_score: {}", score, next_score);
}

#[test]
fn find_signature_test() {
    let html = Html::parse_document(SHOP_PAGE);

    let selector = Selector::parse("div.col:nth-child(1)").unwrap();
    let el = html.select(&selector).next().unwrap();

    let signature = compute_signature(el);

    let matches = find_signature_matches(html.root_element(), &vec![("group", &signature)]);
    let groups = matches["group"].as_slice();
    assert_eq!(groups.len(), 8);

    for scored_el in groups.iter() {
        let el = scored_el.1;
        assert_eq!(el.value().name(), "div");
        assert_eq!(el.attr("class"), Some("col mb-5"));
    }


    let best_matching = find_best_signature_matches(html.root_element(), &vec![("first_item", &signature)]);

    assert_eq!(best_matching["first_item"].1, el);
}

