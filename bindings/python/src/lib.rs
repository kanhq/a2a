use std::sync::OnceLock;

use pyo3::{prelude::*, types::PyDict};

mod value_to_dict;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn load_config<'py>(py: Python<'py>, conf_dir: &str) -> PyResult<Bound<'py, PyAny>> {
  a2a_tojson::load_conf_dir(conf_dir)
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{:?}", e)))
    .and_then(|config| {
      let dict = value_to_dict::value_to_py(py, &config);
      Ok(dict)
    })
}

#[pyfunction]
fn do_action<'py>(py: Python<'py>, action: &Bound<'py, PyDict>) -> PyResult<Bound<'py, PyAny>> {
  let rt = action_runtime();

  let action = value_to_dict::value_from_py(py, action.clone().into_any())?;
  let action = serde_json::from_value(action)
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{:?}", e)))?;
  let result = rt.block_on(async move {
    a2a_core::do_action(action)
      .await
      .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{:?}", e)))
  });

  result.map(|r| value_to_dict::value_to_py(py, &r))
}

// #[pyfunction]
// fn a2a(a: usize, b: usize) -> PyResult<String> {
//   Ok((a + b).to_string())
// }

/// A Python module implemented in Rust.
#[pymodule]
fn a2a(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_function(wrap_pyfunction_bound!(load_config, m)?)?;
  m.add_function(wrap_pyfunction_bound!(do_action, m)?)?;
  Ok(())
}

fn action_runtime() -> &'static tokio::runtime::Runtime {
  static ACTION_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
  ACTION_RUNTIME.get_or_init(|| {
    tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .expect("Failed to create action runtime")
  })
}
