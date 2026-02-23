use std::collections::HashMap;

use furas::model::{FieldSelector, extract_fields, generate_model_str};
use scraper::Html;


const SHOP_PAGE: &str = include_str!("./assets/shop.html");

#[test]
fn test_create_extract_model() {
    let fields = HashMap::from([
        ("image".to_string(), FieldSelector::Selector("div.col:nth-child(1) > div:nth-child(1) > img:nth-child(1)".to_string())),
        ("name".to_string(), FieldSelector::Selector("div.col:nth-child(1) > div:nth-child(1) > div:nth-child(2) > div:nth-child(1) > h5:nth-child(1)".to_string())),
        ("description".to_string(), FieldSelector::Selector("div.col:nth-child(1) > div:nth-child(1) > div:nth-child(2) > div:nth-child(1)".to_string())),
    ]);

    let res = generate_model_str(SHOP_PAGE, &fields);
    assert!(res.is_ok());
    // dbg!(res);

    let model = res.unwrap();

    let html = Html::parse_document(SHOP_PAGE);
    let result = extract_fields(&model, &html).unwrap();

    dbg!(&result);

    assert_eq!(8, result.data.len())
}