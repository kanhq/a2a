use crate::{FromJsonValue, ToJsonValue};
use crate::{Result, Value};

const HEX_PREFIX: &str = "data:";

impl ToJsonValue for &[u8] {
  fn to_json(&self, conf: Option<&Value>) -> Result<Value> {
    let mimetype = conf
      .and_then(|conf| conf.get("mimetype").and_then(Value::as_str))
      .unwrap_or("");
    let mut blob = format!("data:{};base64,", mimetype);
    base64_simd::STANDARD.encode_append(self, &mut blob);
    Ok(Value::String(blob))
  }
}

impl FromJsonValue for Vec<u8> {
  fn from_json(value: &Value) -> Result<Self> {
    let s = value
      .as_str()
      .ok_or_else(|| anyhow::anyhow!("Not a string"))?;
    if s.starts_with(HEX_PREFIX) {
      let pos = s
        .find(',')
        .ok_or_else(|| anyhow::anyhow!("Not a data url"))?;
      let s = &s[pos + 1..];
      base64_simd::STANDARD
        .decode_to_vec(s)
        .map_err(|err| err.into())
    } else {
      Err(anyhow::anyhow!("Not a data url"))
    }
  }
}
