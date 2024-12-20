use std::{
  future::Future,
  sync::{Arc, OnceLock, RwLock},
};

use a2a_core::do_action;
use a2a_types::{Action, Value};
use anyhow::Result;
use futures::{pin_mut, FutureExt};
use quickjs_rusty::{
  serde::{from_js, to_js},
  Arguments, Context, OwnedJsValue,
};
use tokio::{
  runtime::{Handle, Runtime},
  task::yield_now,
};
use tracing::{debug, info};

use crate::{app_conf::Runner, config_loader::load_conf_dir};

pub(crate) async fn execute(arg: &Runner) -> Result<Value> {
  let conf = load_conf_dir(&arg.conf_dir)?;
  let clean_up = arg.clean.clone();
  info!(script=%arg.file, "execute script");
  execute_js_file(&arg.file, &conf, &Value::Null, clean_up).await
}
pub(crate) async fn execute_js_file(
  filename: &str,
  conf: &Value,
  params: &Value,
  clean_up: Option<String>,
) -> Result<Value> {
  let code = std::fs::read_to_string(filename)?;

  execute_js_code(&code, conf, params, clean_up).await
}

pub(crate) async fn execute_js_code(
  code: &str,
  conf: &Value,
  params: &Value,
  clean_up: Option<String>,
) -> Result<Value> {
  let code = code.replace("export", "");

  let log_lines = Arc::new(RwLock::new(Vec::new()));

  let log_console = log::LogConsole::new_with_lines(log_lines.clone());

  let js_ctx = Context::builder().console(log_console).build()?;

  let ctx = unsafe { js_ctx.context_raw() };

  let p_config = to_js(ctx, conf)?;
  let p_params = to_js(ctx, params)?;

  // let js_do_action = js_ctx.create_callback(do_action_quickjs)?;
  // js_ctx.set_global("doAction", js_do_action)?;

  js_ctx.add_callback("doAction", do_action_quickjs)?;

  js_ctx
    .eval(&code, true)
    .map_err(|err| anyhow::anyhow!(err.to_string()))?;

  let result = js_ctx
    .call_function("main", vec![p_config.clone(), p_params.clone()])
    .map_err(|err| anyhow::anyhow!(err.to_string()));

  if let Some(clean_up) = clean_up {
    if let Ok(clean_code) = std::fs::read_to_string(clean_up) {
      js_ctx.set_global("config", p_config.clone())?;
      js_ctx.set_global("params", p_params.clone())?;
      if let Err(err) = js_ctx
        .eval_module(&clean_code, true)
        .map_err(|err| anyhow::anyhow!(err.to_string()))
      {
        debug!("clean up error: {}", err);
      }
    }
  }
  let result: Value =
    result.and_then(|r| from_js(ctx, &r).map_err(|err| anyhow::anyhow!(err.to_string())))?;
  let logs = match log_lines.read() {
    Ok(lines) => lines.clone(),
    Err(_) => Vec::new(),
  };
  let body = serde_json::json!({ "result": result, "logs": logs });
  Ok(body.into())
}

fn do_action_quickjs(args: Arguments) -> Result<OwnedJsValue, String> {
  let mut args = args.into_vec();
  if args.len() != 1 {
    return Err("action should have only one argument".to_string());
  }
  let arg = args.pop().unwrap();

  let action = from_js(arg.context(), &arg).map_err(|err| format!("invalid js action: {}", err))?;

  let action: Action =
    serde_json::from_value(action).map_err(|err| format!("invalid action: {}", err))?;

  let res = tokio::task::block_in_place(move || {
    Handle::current().block_on(async move { do_action(action).await })
  });

  let res = match res {
    Ok(val) => to_js(arg.context(), &val).map_err(|err| err.to_string()),
    //Ok(Err(err)) => Err(err.to_string()),
    Err(err) => Err(err.to_string()),
  };

  res
}

fn action_runtime() -> &'static tokio::runtime::Runtime {
  static ACTION_RUNTIME: OnceLock<Runtime> = OnceLock::new();
  ACTION_RUNTIME.get_or_init(|| {
    tokio::runtime::Builder::new_multi_thread()
      .worker_threads(4)
      .enable_all()
      .build()
      .expect("create action runtime failed")
  })
}

// fn action_runtime() -> Result<Runtime, String> {
//   tokio::runtime::Builder::new_current_thread()
//     .enable_all()
//     .build()
//     .map_err(|err| err.to_string())
// }

mod log {
  use std::{
    panic::RefUnwindSafe,
    sync::{Arc, RwLock},
  };

  use quickjs_rusty::{
    console::{ConsoleBackend, Level},
    OwnedJsValue,
  };
  use tracing::{debug, error, info, trace, warn};

  /// A console implementation that logs messages via the `log` crate.
  ///
  /// Only available with the `log` feature.
  pub struct LogConsole {
    lines: Arc<RwLock<Vec<String>>>,
    enable_lines: bool,
  }

  impl LogConsole {
    pub fn new_with_lines(lines: Arc<RwLock<Vec<String>>>) -> Self {
      Self {
        lines,
        enable_lines: true,
      }
    }
  }

  impl RefUnwindSafe for LogConsole {}

  impl ConsoleBackend for LogConsole {
    fn log(&self, level: Level, values: Vec<OwnedJsValue>) {
      if values.is_empty() {
        return;
      }

      let msg = values
        .iter()
        .map(|v| {
          v.to_string()
            .unwrap_or(v.to_json_string(0).unwrap_or_default())
        })
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

      if self.enable_lines {
        self
          .lines
          .write()
          .map(|mut lines| {
            lines.push(msg);
          })
          .unwrap_or_else(|err| {
            error!("add log error: {}", err);
          });
      }
    }
  }
}
