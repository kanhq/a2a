use a2a_types::{EncAction, EncActionResult};
use anyhow::Result;

mod encode;
mod hash;
mod hmacs;
mod encrypt;


pub(crate) fn do_action(action: EncAction) -> Result<EncActionResult> {
  let mut input_data = parse_data(&action.data)?;
  let key = match action.key {
    Some(key) => Some(parse_data(&key)?),
    None => None,
  };
  let key = key.as_deref();
  let is_enc = !(action.is_dec.unwrap_or(false));
  let padding = action.padding.unwrap_or_default().to_lowercase();

  for method in &action.methods {
    let method = method.to_lowercase();
    input_data = match method.as_str() {
      "base64" => encode::base64_action(is_enc, &input_data, key, &padding), 
      "base64url" => encode::base64url_action(is_enc, &input_data, key, &padding), 
      "hex" => encode::hex_action(is_enc, &input_data, key, &padding),
      "url" => encode::url_action(is_enc, &input_data, key, &padding),
      "md5" => hash::md5_action(is_enc, &input_data, key, &padding),
      "sha1" => hash::sha1_action(is_enc, &input_data, key, &padding),
      "sha1prng" => hash::sha1prng_action(is_enc, &input_data, key, &padding),
      "sha256" => hash::sha256_action(is_enc, &input_data, key, &padding),
      "hmac_md5" => hmacs::hmac_md5_action(is_enc, &input_data, key, &padding),
      "hmac_sha1" => hmacs::hmac_sha1_action(is_enc, &input_data, key, &padding),
      "hmac_sha256" => hmacs::hmac_sha256_action(is_enc, &input_data, key, &padding),
      "aes_ecb" => encrypt::aes_ecb_action(is_enc, &input_data, key, &padding),
      "aes_cbc" => encrypt::aes_cbc_action(is_enc, &input_data, key, &padding),
      _ => anyhow::bail!("Unsupported method: {}", method),
    }?;
  }

  let out = match String::from_utf8(input_data) {
    Ok(data) => data,
    Err(e) => {
      let mut blob = format!("data:application/octet-stream;base64,");
      base64_simd::STANDARD.encode_append(e.as_bytes(), &mut blob);
      blob
    },
  };
  Ok(out)
}


fn parse_data<S: AsRef<str>>(data: S) -> Result<Vec<u8>> {
  // check if data is data url
  let data = data.as_ref();
  if data.starts_with("data:") {
    let parts: Vec<&str> = data.splitn(2, ",").collect();
    if parts.len() != 2 {
      anyhow::bail!("Invalid data url");
    }
    let data = parts[1];
    base64_simd::STANDARD.decode_to_vec(data).map_err(Into::into)
  } else {
    Ok(data.as_bytes().to_vec())
  }
}