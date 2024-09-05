use a2a_core::do_action;
use a2a_types::{Action, Value};
use anyhow::Result;
use quick_js::{Arguments, JsValue};
use serde_json::json;

pub(crate) async fn execute_js(filename: &str, conf: Value) -> Result<Value> {
  let code = std::fs::read_to_string(filename)?;

  let js_ctx = quick_js::Context::builder()
    .console(log::LogConsole)
    .build()?;

  js_ctx.set_global("conf", to_js_value(conf))?;
  js_ctx.add_callback("doAction", do_action_quickjs)?;

  js_ctx.eval(&code)?;

  Ok(Value::Null)
}

fn to_js_value(val: Value) -> JsValue {
  match val {
    Value::Null => JsValue::Null,
    Value::Bool(b) => JsValue::Bool(b),
    Value::Number(n) => {
      if n.is_f64() {
        JsValue::Float(n.as_f64().unwrap_or_default())
      } else {
        JsValue::BigInt(n.as_i64().unwrap_or_default().into())
      }
    }
    Value::String(s) => JsValue::String(s),
    Value::Array(arr) => {
      let mut js_arr = Vec::new();
      for val in arr {
        js_arr.push(to_js_value(val));
      }
      JsValue::Array(js_arr)
    }
    Value::Object(obj) => {
      let mut js_obj = std::collections::HashMap::new();
      for (key, val) in obj {
        js_obj.insert(key, to_js_value(val));
      }
      JsValue::Object(js_obj)
    }
  }
}

fn from_js_value(val: JsValue) -> Value {
  match val {
    JsValue::Null => Value::Null,
    JsValue::Bool(b) => Value::Bool(b),
    JsValue::Int(n) => Value::Number(n.into()),
    JsValue::BigInt(n) => json!(n.as_i64().unwrap_or_default()),
    JsValue::Float(f) => json!(f),
    JsValue::String(s) => Value::String(s),
    JsValue::Array(arr) => {
      let mut js_arr = Vec::new();
      for val in arr {
        js_arr.push(from_js_value(val));
      }
      Value::Array(js_arr)
    }
    JsValue::Object(obj) => {
      let mut js_obj = serde_json::Map::new();
      for (key, val) in obj {
        js_obj.insert(key, from_js_value(val));
      }
      Value::Object(js_obj)
    }
    _ => Value::Null,
  }
}

fn do_action_quickjs(args: Arguments) -> Result<JsValue, String> {
  let mut args = args.into_vec();
  if args.len() != 1 {
    return Err("action should have only one argument".to_string());
  }
  let action = from_js_value(args.pop().unwrap());

  let action: Action =
    serde_json::from_value(action).map_err(|err| format!("invalid action: {}", err))?;

  //let rt = action_runtime();
  futures::executor::block_on(async move {
    let res = do_action(action).await;
    match res {
      Ok(val) => Ok(to_js_value(val)),
      Err(err) => Err(err.to_string()),
    }
  })
}

mod log {
  use quick_js::{
    console::{ConsoleBackend, Level},
    JsValue,
  };
  use tracing::{debug, error, info, trace, warn};

  /// A console implementation that logs messages via the `log` crate.
  ///
  /// Only available with the `log` feature.
  pub struct LogConsole;

  fn print_value(value: JsValue) -> String {
    match value {
      JsValue::Undefined => "undefined".to_string(),
      JsValue::Null => "null".to_string(),
      JsValue::Bool(v) => v.to_string(),
      JsValue::Int(v) => v.to_string(),
      JsValue::Float(v) => v.to_string(),
      JsValue::String(v) => v,
      JsValue::Array(values) => {
        let parts = values
          .into_iter()
          .map(print_value)
          .collect::<Vec<_>>()
          .join(", ");
        format!("[{}]", parts)
      }
      JsValue::Object(map) => {
        let parts = map
          .into_iter()
          .map(|(key, value)| format!("{}: {}", key, print_value(value)))
          .collect::<Vec<_>>()
          .join(", ");
        format!("{{{}}}", parts)
      }
      JsValue::BigInt(v) => v.to_string(),
      JsValue::__NonExhaustive => unreachable!(),
    }
  }

  impl ConsoleBackend for LogConsole {
    fn log(&self, level: Level, values: Vec<JsValue>) {
      if values.is_empty() {
        return;
      }

      let msg = values
        .into_iter()
        .map(print_value)
        .collect::<Vec<_>>()
        .join(" ");

      match level {
        Level::Trace => trace!("{}", msg),
        Level::Debug => debug!("{}", msg),
        Level::Log => info!("{}", msg),
        Level::Info => info!("{}", msg),
        Level::Warn => warn!("{}", msg),
        Level::Error => error!("{}", msg),
      };
    }
  }
}
