use scraper::{Element, ElementRef, Html, Selector};


const SHOP_PAGE: &str = include_str!("./assets/shop.html");

fn select<'a>(html: &'a Html, selector: &str) -> ElementRef<'a> {
    let selector = Selector::parse(selector).unwrap();

    html.select(&selector).next().unwrap()   
}

#[test]
fn lca_test() {
    let html = Html::parse_document(SHOP_PAGE);

    let el1 = select(&html, "div.col:nth-child(1) > div:nth-child(1) > div:nth-child(2) > div:nth-child(1) > h5:nth-child(1)");
    let el2 = select(&html, "div.col:nth-child(1) > div:nth-child(1) > img:nth-child(1)");
    let el3 = select(&html, "div.col:nth-child(1) > div:nth-child(1) > div:nth-child(3) > div:nth-child(1) > a:nth-child(1)");

    let parent = select(&html, "div.col:nth-child(1) > div:nth-child(1)");
    let parent_of_parent = parent.parent_element().unwrap();

    assert_eq!(furas::utils::lca(&[el1, el2]), Some(parent));
    assert_eq!(furas::utils::lca(&[el1, el2, el3]), Some(parent));
    assert_ne!(furas::utils::lca(&[el1, el2, el3]), Some(parent_of_parent));

    let el4 = select(&html, "head > link:nth-child(8)");
    let el5 = select(&html, "footer.py-5");

    assert_eq!(furas::utils::lca(&[el4, el5]).unwrap(), select(&html, "html"))
}

