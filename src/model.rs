// use core::fmt;d
use pyo3::prelude::*;
use std::collections::HashMap;

use itertools::Itertools;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

use crate::{
    furas::{
        ScoredElement, Signature, compute_signature, find_best_signature_matches,
        find_signature_matches,
    },
    utils::lca,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct NormalisationOptions {
    unwrap_text_tags: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FieldSignature {
    Signature(Signature),
    Submodel(Model),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    /// Options for normalising html
    pub normalisation: NormalisationOptions,

    /// Signature of lowest common ancestor of all elements
    pub group_signature: Signature,

    /// Signatures of all fields of this model
    pub field_signatures: HashMap<String, FieldSignature>,
}

#[derive(Clone)]
pub enum FieldSelector {
    Selector(String),
    Submodel(FieldSelectors),
}
pub type FieldSelectors = HashMap<String, FieldSelector>;

pub fn generate_model_str(html: &str, selectors: &FieldSelectors) -> Result<Model, String> {
    let document = Html::parse_document(html);

    let root = document.root_element();

    generate_model(root, selectors)
}

pub fn generate_model(root: ElementRef, selectors: &FieldSelectors) -> Result<Model, String> {
    let mut signatures = HashMap::new();

    // compute signatures for all elements
    for (field, field_selector) in selectors.iter() {
        let sig: FieldSignature = match field_selector {
            FieldSelector::Selector(s) => {
                let selector = Selector::parse(s);
                if let Err(err) = selector {
                    return Err(err.to_string());
                }

                let el = root.select(&selector.unwrap()).next();

                if el.is_none() {
                    return Err(format!(
                        "Could not find element for field {} using selector {}",
                        field, s
                    ));
                }

                FieldSignature::Signature(compute_signature(el.unwrap()))
            }
            FieldSelector::Submodel(selectors) => {
                let m = generate_model(root, selectors);
                if let Err(err) = m {
                    return Err(err);
                }

                FieldSignature::Submodel(m.unwrap())
            }
        };

        signatures.insert(field.clone(), sig);
    }

    // compute signature for group
    let field_signature_list = signatures
        .iter()
        .by_ref()
        .map(|(field, signature)| match signature {
            FieldSignature::Signature(s) => (field.as_str(), s),
            FieldSignature::Submodel(m) => (field.as_str(), &m.group_signature),
        })
        .collect_vec();

    let best_matches = find_best_signature_matches(root, &field_signature_list);

    let best_matches_elements: Vec<ElementRef> =
        best_matches.values().map(|scored_el| scored_el.1).collect();

    let group_parent = lca(&best_matches_elements);
    if group_parent.is_none() {
        return Err("Could not find group for all elements.".to_string());
    }

    let group_signaure = compute_signature(group_parent.unwrap());

    Ok(Model {
        normalisation: NormalisationOptions {
            unwrap_text_tags: false,
        },
        field_signatures: signatures,
        group_signature: group_signaure,
    })
}

// #[derive(Debug)]
// pub struct Match<'a> {
//     pub value: MatchValue<'a>,
//     pub score: f64,
// }

#[derive(Debug)]
pub enum MatchValue<'a> {
    Element(ElementRef<'a>),
    Group(HashMap<String, Option<MatchValue<'a>>>),
    List(Vec<MatchValue<'a>>)
}

#[derive(Debug)]
pub struct GroupScore {
    pub score: f64,
    pub fields: HashMap<String, Option<MatchScore>>
}

#[derive(Debug)]
pub enum MatchScore {
    Element(f64),
    Group(GroupScore),
    List(Vec<MatchScore>)
}

// #[derive(Debug)]
// pub enum FieldValue<'a> {
//     Element(Match<'a>),
//     // Group(ModelFields<'a>),
//     Group(Vec<HashMap<String, Option<FieldValue<'a>>>>),
// }

// type ModelFields<'a> = HashMap<String, Option<FieldValue<'a>>>;

#[derive(Debug)]
pub struct ExtractResult<'a> {
    pub scores: Vec<MatchScore>,
    pub data: Vec<MatchValue<'a>>,
}

pub fn extract_fields<'a, 'b>(
    model: &'b Model,
    html: &'a Html,
) -> Result<ExtractResult<'a>, String>
where
    'b: 'a,
{
    let groups = find_signature_matches(html.root_element(), &[("root", &model.group_signature)]);

    let root = groups.get("root");
    if root.is_none() || root.unwrap().is_empty() {
        return Err("could not find model group".to_string());
    }

    let mut group_fields = Vec::with_capacity(root.unwrap().len());
    let mut group_scores = Vec::with_capacity(root.unwrap().len());

    for ScoredElement(group_score, el) in root.unwrap() {
        let signatures = model
            .field_signatures
            .iter()
            .filter_map(|(f, sig)| match sig {
                FieldSignature::Signature(s) => Some((f.as_str(), s)),
                FieldSignature::Submodel(_) => None,
            })
            .collect_vec();

        let submodels = model
            .field_signatures
            .iter()
            .filter_map(|(f, sig)| match sig {
                FieldSignature::Signature(_) => None,
                FieldSignature::Submodel(m) => Some((f.as_str(), m)),
            })
            .collect_vec();

        let fields: HashMap<&str, ScoredElement<'_>> =
            find_best_signature_matches(*el, &signatures);
        let model_fields = submodels
            .iter()
            .map(|(f, m)| (*f, extract_fields(m, html).ok()))
            .collect_vec();

        let mut all_fields: HashMap<String, Option<MatchValue>> = HashMap::with_capacity(fields.len() + model_fields.len());
        let mut all_scores: HashMap<String, Option<MatchScore>> = HashMap::with_capacity(fields.len() + model_fields.len());

        for (f, scored_el) in fields {
            all_fields.insert(
                f.to_string(),
                Some(MatchValue::Element(scored_el.1)),
            );
            all_scores.insert(
                f.to_string(),
                Some(MatchScore::Element(scored_el.0)),
            );
        }

        for (f, result) in model_fields {
            match result {
                Some(res) => {
                    all_fields.insert(f.to_string(), Some(MatchValue::List(res.data)));
                    all_scores.insert(f.to_string(), Some(MatchScore::List(res.scores)));
                },
                None => {
                    all_fields.insert(f.to_string(), None);
                    all_scores.insert(f.to_string(), None);
                }
            };
        }
        group_fields.push(MatchValue::Group(all_fields));
        group_scores.push(MatchScore::Group(GroupScore { score: *group_score, fields: all_scores }));
    }

    Ok(ExtractResult { scores: (group_scores), data: (group_fields) })
}
