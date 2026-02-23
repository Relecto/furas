use std::sync::OnceLock;
use std::{collections::HashMap, str::FromStr};

use regex::Regex;
use scraper::ElementRef;
use serde::{Deserialize, Serialize};

use itertools::Itertools;

use crate::utils::{self, index_in_parent};

use crate::math::{softmax_diff};

#[derive(Serialize, Deserialize, Debug)]
pub enum AttrValue {
    Signle(String),
    List(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Signature {
    pub tag_name: String,
    pub attrs: HashMap<String, AttrValue>,
    pub depth: u32,
    pub has_children: bool,
    pub index_in_parent: u32,
    pub text_len: u32,
    pub regex: Option<String>,
    pub weights: HashMap<String, f64>,
    pub meta: HashMap<String, String>,
 
    #[serde(skip_serializing, skip_deserializing)]
    compiled_regex: OnceLock<Option<Regex>>,
}

impl Signature {
    pub fn get_regex(&self) -> Option<&Regex> {
        self.compiled_regex.get_or_init(|| {
            self.regex.as_ref()?;

            Regex::new(self.regex.as_ref().unwrap()).ok()
        }).as_ref()
    }
}

/// Split value by whitespace and sort lexicographically.
/// Return the result as vec.
fn normalise_attr_value(attr_val: &str) -> Vec<String> {
    attr_val
        .split_whitespace()
        .sorted()
        .map(|s| s.to_string())
        .collect()
}

fn element_text_len(el: ElementRef) -> usize {
    el.text().fold(0, |acc, e| acc + e.len())
}

pub fn compute_signature(el: ElementRef) -> Signature {
    Signature {
        tag_name: String::from_str(el.value().name()).unwrap(),
        attrs: el
            .value()
            .attrs()
            .map(|(k, v)| (
                k.to_string(), 
                AttrValue::List(normalise_attr_value(v))
            ))
            .collect(),
        depth: { // scraper adds a parent "document" node, so let's strip it.
            let anc_count = el.ancestors().count() as u32;
            if anc_count == 0 {
                0
            } else {
                anc_count - 1
            }
        },
        has_children: el.has_children(),
        index_in_parent: index_in_parent(el) as u32,
        text_len: element_text_len(el) as u32,
        regex: None,
        weights: HashMap::new(),
        meta: HashMap::new(),

        compiled_regex: OnceLock::new(),
    }
}

pub fn compare_signature(target: &Signature, candidate: ElementRef) -> f64 {
    let candidate_signature = compute_signature(candidate);
    let el = candidate.value();

    // vector stores a tuple of (score, weight) for each criteria
    let mut scores = Vec::<(f64, f64)>::new();

    let el_text = candidate.text().join(" ");

    // first, check critera that completely disqualify an element
    // TODO this can be remodelled as 0 weight on remaining criteria
    if let Some(regex) = target.get_regex() {
        if !regex.is_match(&el_text) {
            return 0.0;
        }
        scores.push((1.0, *target.weights.get("regex").unwrap_or(&1.0)));
    }

    // check the rest of the criteria
    // tag name match
    scores.push((
        utils::compare_html_tag_name(el.name(), &target.tag_name),
        *target.weights.get("tag_name").unwrap_or(&1.0),
    ));

    // attribute match
    for (attr, value) in &target.attrs {
        let expected_values = match value {
            AttrValue::Signle(s) => &vec![s.clone()],
            AttrValue::List(v) => v
        };
        
        let attr_values = match candidate.attr(attr) {
            None => vec![],
            Some(val) => normalise_attr_value(val)
        };

        for expected_val in expected_values {
            let mut max = 0.0;
            for val in &attr_values {
                let score = strsim::normalized_damerau_levenshtein(expected_val, val);
                if score > max {
                    max = score;
                }
            }
            scores.push((max, *target.weights.get("attrs").unwrap_or(&1.0)))
        }
    }

    // tree depth score
    let depth_diff = u32::abs_diff(target.depth, candidate_signature.depth);
    let depth_score = f64::clamp(1.0 - (depth_diff as f64 * 0.2), 0.0, 1.0) ;
    scores.push((
        depth_score, *target.weights.get("depth").unwrap_or(&1.0)
    ));


    // text len score
    let text_len_ratio = softmax_diff(0.2, target.text_len as f64, candidate_signature.text_len as f64);
    scores.push((
        text_len_ratio, *target.weights.get("text_len").unwrap_or(&1.0)
    ));
    // println!("text len a b {} {}", target.text_len, candidate_signature.text_len);
    // println!("text len ratio {}", text_len_ratio);

    // println!("scores: {:?}", scores);

    // compute total score
    let mut total_score = 0f64;
    let mut total_weight = 0f64;

    for (score, weight) in scores {
        total_score += score * weight;
        total_weight += weight;
    }
    total_score / total_weight
}

#[derive(Debug)]
pub struct ScoredElement<'a>(pub f64, pub ElementRef<'a>);

pub fn find_signature_matches<'a>(root: ElementRef<'a>, signatures: &[(&'a str, &'a Signature)]) -> HashMap<&'a str, Vec<ScoredElement<'a>>> {
    let mut matches = HashMap::with_capacity(signatures.len());

    for element in root.descendent_elements() {
        for (field, signature) in signatures {
            let score = compare_signature(signature, element);

            if score >= 0.75 {
                matches.entry(*field).or_insert(Vec::new()).push(ScoredElement(score, element));
            }

        }
    }

    matches
}

pub fn find_best_signature_matches<'a>(root: ElementRef<'a>, signatures: &[(&'a str, &'a Signature)]) -> HashMap<&'a str, ScoredElement<'a>> {
    let mut matches = HashMap::with_capacity(signatures.len());

    for element in root.descendent_elements() {
        for (field, signature) in signatures.iter() {
            let score = compare_signature(signature, element);
            matches.entry(*field)
                .and_modify(|e: &mut ScoredElement<'_>| if score > e.0 { e.0 = score; e.1 = element })
                .or_insert(ScoredElement(score, element));
        }
    }

    matches
}