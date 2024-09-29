use a2a_core::do_action;
use a2a_types::{Action, Value};
use anyhow::Result;
use quickjs_rusty::{
  serde::{from_js, to_js},
  Arguments, Context, OwnedJsValue,
};
use tracing::{debug, info};

use crate::{app_conf::Runner, config_loader::load_conf_dir};

pub(crate) async fn execute(arg: &Runner) -> Result<Value> {
  let conf = load_conf_dir(&arg.conf_dir)?;
  let clean_up = arg.clean.clone();
  info!(script=%arg.file, "execute script");
  execute_js(&arg.file, &conf, &Value::Null, clean_up).await
}

pub(crate) async fn execute_js(
  filename: &str,
  conf: &Value,
  params: &Value,
  clean_up: Option<String>,
) -> Result<Value> {
  let code = std::fs::read_to_string(filename)?.replace("export", "");

  let js_ctx = Context::builder().console(log::LogConsole).build()?;

  let ctx = unsafe { js_ctx.context_raw() };

  let p_config = to_js(ctx, conf)?;
  let p_params = to_js(ctx, params)?;

  let js_do_action = js_ctx.create_callback(do_action_quickjs)?;
  js_ctx.set_global("doAction", js_do_action)?;

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
  result.and_then(|r| from_js(ctx, &r).map_err(|err| anyhow::anyhow!(err.to_string())))
}

fn do_action_quickjs(args: Arguments) -> Result<OwnedJsValue, String> {
  let mut args = args.into_vec();
  if args.len() != 1 {
    return Err("action should have only one argument".to_string());
  }
  let arg = args.pop().unwrap();

  let action = from_js(arg.context(), &arg).map_err(|err| format!("invalid action: {}", err))?;

  let action: Action =
    serde_json::from_value(action).map_err(|err| format!("invalid action: {}", err))?;

  //let rt = action_runtime();
  let res = futures::executor::block_on(async move {
    let res = do_action(action).await;
    match res {
      Ok(val) => to_js(arg.context(), &val).map_err(|err| err.to_string()),
      Err(err) => Err(err.to_string()),
    }
  });

  res
}

mod log {
  use quickjs_rusty::{
    console::{ConsoleBackend, Level},
    OwnedJsValue,
  };
  use tracing::{debug, error, info, trace, warn};

  /// A console implementation that logs messages via the `log` crate.
  ///
  /// Only available with the `log` feature.
  pub struct LogConsole;

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
    }
  }
}
