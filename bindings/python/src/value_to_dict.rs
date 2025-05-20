use std::collections::HashMap;

use a2a_types::Value;
use anyhow::Ok;
use pyo3::{
  prelude::*,
  types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyNone, PyString},
  IntoPyObjectExt,
};
use serde_json::{Map, Number};

pub(crate) fn value_to_py<'py>(py: Python<'py>, value: &Value) -> PyResult<Bound<'py, PyAny>> {
  match value {
    Value::Number(n) => {
      if n.is_f64() {
        n.as_f64().unwrap_or_default().into_bound_py_any(py)
      } else {
        n.as_i64().unwrap_or_default().into_bound_py_any(py)
      }
    }
    Value::String(s) => s.into_bound_py_any(py),
    Value::Bool(b) => b.into_bound_py_any(py),
    Value::Array(a) => a
      .iter()
      .map_while(|v| value_to_py(py, v).ok())
      .collect::<Vec<_>>()
      .into_bound_py_any(py),
    Value::Object(o) => o
      .iter()
      .map_while(|(k, v)| value_to_py(py, v).ok().map(|v| (k, v)))
      .collect::<HashMap<_, _>>()
      .into_bound_py_any(py),
    Value::Null => py.None().into_bound_py_any(py),
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
  let v = value.downcast_exact::<PyInt>();
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
  let v = value.downcast::<PyString>();
  if v.is_ok() {
    let v = v.unwrap().extract::<String>()?;
    return Ok(Value::String(v));
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

  Err(anyhow::anyhow!("Unsupported type {:?}", value.get_type()))
}

pub(crate) struct ValuePy {
  pub(crate) value: Value,
}

impl<'py> IntoPyObject<'py> for ValuePy {
  type Target = PyAny; // the Python type
  type Output = Bound<'py, Self::Target>; // in most cases this will be `Bound`
  type Error = PyErr;

  fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
    value_to_py(py, &self.value)
  }
}
