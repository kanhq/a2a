use std::collections::HashMap;

use a2a_types::Value;
use anyhow::Ok;
use pyo3::{
  types::{PyAnyMethods, PyBool, PyBoolMethods, PyDict, PyFloat, PyList, PyLong, PyNone},
  Bound, IntoPy, PyAny, Python,
};
use serde_json::{Map, Number};

pub(crate) fn value_to_py<'py>(py: Python<'py>, value: &Value) -> Bound<'py, PyAny> {
  match value {
    Value::Number(n) => {
      if n.is_f64() {
        n.as_f64().unwrap_or_default().into_py(py).into_bound(py)
      } else {
        n.as_i64().unwrap_or_default().into_py(py).into_bound(py)
      }
    }
    Value::String(s) => s.into_py(py).into_bound(py),
    Value::Bool(b) => b.into_py(py).into_bound(py),
    Value::Array(a) => a
      .iter()
      .map(|v| value_to_py(py, v))
      .collect::<Vec<_>>()
      .into_py(py)
      .into_bound(py),
    Value::Object(o) => o
      .iter()
      .map(|(k, v)| (k.clone(), value_to_py(py, v)))
      .collect::<HashMap<_, _>>()
      .into_py(py)
      .into_bound(py),
    Value::Null => py.None().into_bound(py),
  }
}

pub(crate) fn value_from_py<'py>(py: Python, value: Bound<'py, PyAny>) -> anyhow::Result<Value> {
  let v = value.downcast_exact::<PyNone>();
  if v.is_ok() {
    return Ok(Value::Null);
  }
  let v = value.downcast_exact::<PyBool>();
  if v.is_ok() {
    return Ok(Value::Bool(v.unwrap().is_true()));
  }
  let v = value.downcast_exact::<PyLong>();
  if v.is_ok() {
    let v = v.unwrap().extract::<i64>().unwrap_or_default();
    return Ok(Value::Number(v.into()));
  }
  let v = value.downcast::<PyFloat>();
  if v.is_ok() {
    let v = v.unwrap().extract::<f64>().unwrap_or_default();
    return Ok(
      Number::from_f64(v)
        .map(Value::Number)
        .unwrap_or(Value::Null),
    );
  }
  let v = value.downcast::<PyList>();
  if v.is_ok() {
    let v = v.unwrap();
    return v
      .into_iter()
      .map(|v| value_from_py(py, v))
      .collect::<Result<Vec<_>, _>>()
      .map(|a| Value::Array(a));
  }
  let v = value.downcast::<PyDict>();
  if v.is_ok() {
    let v = v.unwrap();
    return v
      .into_iter()
      .map(|(k, v)| {
        let k = k.extract::<String>()?;
        let v = value_from_py(py, v)?;
        Ok((k, v))
      })
      .collect::<Result<Map<_, _>, _>>()
      .map(|m| Value::Object(m));
  }

  Err(anyhow::anyhow!("Unsupported type"))
}
