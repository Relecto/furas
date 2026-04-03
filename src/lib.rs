use pyo3::prelude::*;
mod math;
pub mod model;
pub mod signature;
pub mod utils;

/// A Python module implemented in Rust.
#[pymodule]
mod furas {

    use std::{collections::HashMap, fmt::Display};

    use crate::model::{FieldSelectors, MatchValue, extract_fields, generate_model_str};
    pub use crate::signature::*;
    use itertools::Itertools;
    use pyo3::{
        exceptions::{PyTypeError, PyValueError},
        prelude::*,
        types::{PyDict, PyList, PyNone},
    };
    use scraper::Html;


    #[pymodule_export]
    use crate::model::{FieldSpec, Model, NormalisationOptions};
    // #[pyclass]
    // pub struct FurasModel(Model);
    #[pymethods]
    impl Model {
        #[staticmethod]
        fn from_json<'py>(py: Python<'py>, json: &str) -> PyResult<Bound<'py, Model>> {
            let parsed = serde_json::from_str::<'_, Model>(json);

            match parsed {
                Ok(model) => Ok(Py::new(py, Model::from(model))?.into_bound(py)),
                Err(err) => Err(PyValueError::new_err(err.to_string())),
            }
        }

        fn to_json<'py>(&self) -> PyResult<String> {
            let serialised = serde_json::to_string::<Model>(&self);
            match serialised {
                Ok(s) => Ok(s),
                Err(err) => Err(PyValueError::new_err(err.to_string())),
            }
        }
    }

    // impl From<Model> for Model {
    //     fn from(m: Model) -> Self {
    //         FurasModel(m)
    //     }
    // }

    #[pyclass(str)]
    pub struct ExtractedEl {
        #[pyo3(get)]
        tag: String,
        #[pyo3(get)]
        attrs: Py<PyDict>,
        #[pyo3(get)]
        text: Vec<String>,
    }

    impl Display for ExtractedEl {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "<{} {}>{}</>", self.tag, self.attrs, self.text.iter().join(" "))
        }
    }

    fn extract_val_to_py<'p>(py: Python<'p>, match_val: &MatchValue) -> Bound<'p, PyAny> {
        match match_val {
            MatchValue::Element(e) => {
                let el = ExtractedEl {
                    tag: e.value().name().to_string(),
                    attrs: {
                        let dict = PyDict::new(py);
                        for attr in e.value().attrs() {
                            dict.set_item(attr.0, attr.1).unwrap();
                        }
                        dict.into()
                    },
                    text: e.text().map(|s| s.to_string()).collect()
                };

                Py::new(py, el).unwrap().into_bound(py).into_any()
            }
            MatchValue::Group(g) => {
                let dict = PyDict::new(py);
                for (k, val) in g.iter() {
                    match val {
                        Some(v) => dict.set_item(k, extract_val_to_py(py, v)).unwrap(),
                        None => dict.set_item(k, PyNone::get(py)).unwrap(),
                    };
                }
                dict.into_any()
            }
            MatchValue::List(l) => {
                let list = PyList::new(py, l.iter().map(|e| extract_val_to_py(py, e))).unwrap();

                list.into_any()
            }
        }
    }

    // #[derive(Clone, FromPyObject)]
    // pub struct FurasFieldSelectors(FieldSelectors);
    //
    fn to_furas_field_selectors(
        selectors: HashMap<String, Bound<'_, PyAny>>,
    ) -> PyResult<FieldSelectors> {
        selectors
            .iter()
            .map(|(k, v)| {
                if let Ok(spec) = v.extract::<FieldSpec>() {
                    PyResult::Ok((k.clone(), spec))
                } else if let Ok(string) = v.extract::<String>() {
                    PyResult::Ok((k.clone(), FieldSpec::Field {
                        selector: string,
                        regex: None
                    } ))
                } else if let Ok(dict) = v.extract::<HashMap<String, Bound<'_, PyAny>>>() {
                    PyResult::Ok((k.clone(), FieldSpec::Submodel {
                        fields: to_furas_field_selectors(dict)?,
                    }))
                } else {
                    PyResult::Err(PyTypeError::new_err(format!(
                        "Expected str,  dict or FieldSpec, found {}",
                        v.get_type().to_string()
                    )))
                }
            })
            .collect()
    }

    #[pyfunction]
    #[pyo3(signature = (html: "str", selectors: "dict[str, FieldSpec | str]"))]
    fn generate_model<'p>(
        py: Python<'p>,
        html: &str,
        selectors: HashMap<String, Bound<'_, PyAny>>,
    ) -> PyResult<Bound<'p, Model>> {
        let res = generate_model_str(html, &to_furas_field_selectors(selectors)?);
        if let Err(err) = res {
            return Err(PyValueError::new_err(err));
        }

        let model: Model = res.unwrap().into();

        Ok(Py::new(py, model).unwrap().into_bound(py))
    }

    #[pyfunction]
    #[pyo3(signature = (model: "Model", html: "str | bytes") -> "dict[str, ExtractedEl | dict[str, ExtractedEl] | list[ExtractedEl]]")]
    fn extract<'p>(
        py: Python<'p>,
        model: PyRef<Model>,
        html: &str,
    ) -> PyResult<Bound<'p, PyAny>> {
        let doc = Html::parse_document(html);

        let res = extract_fields(&model, &doc);
        if let Err(e) = res {
            return Err(PyValueError::new_err(e));
        }

        // let collected_data = .collect();

        Ok(PyList::new(
            py,
            res.unwrap().data.iter().map(|m| extract_val_to_py(py, m)),
        )
        .unwrap()
        .into_any())
    }
    /// Formats the sum of two numbers as string.
    #[pyfunction]
    fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
        Ok((a + b).to_string())
    }
}
