use anyhow::Result;


pub(crate) fn base64_action(is_enc: bool, input_data: &[u8], _key: Option<&[u8]>, _padding: &str) -> Result<Vec<u8>> {
  let base64 = base64_simd::STANDARD;
  if is_enc {
    let out : Vec<u8> = base64.encode_type(input_data);
    Ok(out)
  } else {
     base64.decode_to_vec(input_data).map_err(Into::into)
  }
}

pub(crate) fn hex_action(is_enc: bool, input_data: &[u8], _key: Option<&[u8]>, _padding: &str) -> Result<Vec<u8>> {
  if is_enc {
    Ok(hex_simd::encode_type(input_data, hex_simd::AsciiCase::Lower))
  } else {
    hex_simd::decode_to_vec(input_data).map_err(Into::into)
  }
}

pub (crate) fn url_action(is_enc: bool, input_data: &[u8], _key: Option<&[u8]>, _padding: &str) -> Result<Vec<u8>> {
  if is_enc {
    Ok(urlencoding::encode_binary(input_data).as_bytes().to_vec())
  } else {
    Ok(urlencoding::decode_binary(input_data).to_vec())
  }
}