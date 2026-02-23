use std::collections::HashSet;
use std::hash::Hash;

use scraper::{Element, ElementRef};

const TEXT_TAGS: &[&str] = &[
    "p", 
    "span", 
    "h1", "h2", "h3", "h4", "h5", "h6",
];

const CONTAINER_TAGS: &[&str] = &[
    "div", "section"
];


/// Compare how similar tag is to target and return match score from 0 to 1.
///
pub fn compare_html_tag_name(tag: &str, target: &str) -> f64 {
    if tag == target {
        return 1.0;
    }

    if TEXT_TAGS.contains(&tag) && TEXT_TAGS.contains(&target) {
        return 0.7;
    }
    if CONTAINER_TAGS.contains(&tag) && CONTAINER_TAGS.contains(&target) {
        return 0.7;
    }    

    0.0
}

/// Compute element's index in parent
pub fn index_in_parent(el: ElementRef) -> usize {
    match el.parent_element() {
        None => 0,
        Some(parent) => parent.child_elements().position(|e: ElementRef<'_>| e == el).unwrap_or(0),
    }
}

pub fn lca<'a>(elements: &[ElementRef<'a>]) -> Option<ElementRef<'a>> {
    if elements.is_empty() {
        return None;
    }

    #[derive(PartialEq, Eq, Clone)]
    struct HashableElementRef<'a>(ElementRef<'a>);
    impl Hash for HashableElementRef<'_> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.0.id().hash(state);
        }
    }

    let wrap = |node| ElementRef::wrap(node).map(HashableElementRef);

    let mut paths = Vec::with_capacity(elements.len());

    for el in elements.iter() {
        #[allow(clippy::mutable_key_type)]
        let mut set = HashSet::new();
        for parent in el.ancestors() {
            if !parent.value().is_element() {
                continue;
            }
            
            set.insert(wrap(parent).unwrap());
        }

        paths.push(set);
    }

    let path_refs = Vec::from_iter(paths.iter());

    #[allow(clippy::mutable_key_type)]
    let common_parents = intersections(&path_refs);

    let deepest_parent = common_parents.iter().reduce(|acc, el| {
        if acc.0.ancestors().count() > el.0.ancestors().count() {
            acc
        } else {
            el
        }
    });

    deepest_parent.map(|p| p.0)
}

// from @burjui 
// https://users.rust-lang.org/t/intersection-of-multiple-hashsets/85318/2
/// Intersect multiple sets
fn intersections<T>(sets: &[&HashSet<T>]) -> HashSet<T>
where
    T: Clone + Eq + Hash,
{
    match sets.len() {
        0 => HashSet::new(),
        _ => sets[1..].iter().fold(sets[0].clone(), |mut acc, set| {
            acc.retain(|item| set.contains(item));
            acc
        }),
    }
}

#[test]
fn test_intersections() {
    let a: HashSet<_> = [1, 2, 3, 4].into();
    let b: HashSet<_> = [2, 3, 4, 5].into();
    let c: HashSet<_> = [3, 4, 5, 6].into();
    assert_eq!(intersections::<i32>(&[]), HashSet::new());
    assert_eq!(intersections(&[&[1].into()]), [1].into());
    assert_eq!(intersections(&[&a, &b, &c]), [3, 4].into());
    assert_eq!(intersections(&[&[1,2].into(), &[3,4].into()]), HashSet::new());
}