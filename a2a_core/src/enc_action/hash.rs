use anyhow::Result;
use md5::{Md5, Digest};
use sha1::Sha1;
use sha2::Sha256;

pub(crate) fn md5_action(is_enc: bool, input_data: &[u8], _key: Option<&[u8]>, _padding: &str) -> Result<Vec<u8>> {
  if is_enc {
    let mut h = Md5::new();
    h.update(input_data);
    let r = h.finalize();
    Ok(r.to_vec())
  } else {
    anyhow::bail!("Decryption not supported for md5")
  }
}

pub(crate) fn sha1_action(is_enc: bool, input_data: &[u8], _key: Option<&[u8]>, _padding: &str) -> Result<Vec<u8>> {
  if is_enc {
    let mut h = Sha1::new();
    h.update(input_data);
    let r = h.finalize();
    Ok(r.to_vec())
  } else {
    anyhow::bail!("Decryption not supported for sha1")
  }
}

pub(crate) fn sha256_action(is_enc: bool, input_data: &[u8], _key: Option<&[u8]>, _padding: &str) -> Result<Vec<u8>> {
  if is_enc {
    let mut h = Sha256::new();
    h.update(input_data);
    let r = h.finalize();
    Ok(r.to_vec())
  } else {
    anyhow::bail!("Decryption not supported for sha256")
  }
}

pub(crate) fn sha1prng_action(is_enc: bool, input_data: &[u8], _key: Option<&[u8]>, _padding: &str) -> Result<Vec<u8>> {
  if is_enc {
    let mut h = Sha1::new();
    h.update(input_data);
    let r = h.finalize();
    let input_data = r.as_slice();
    let mut h = Sha1::new();
    h.update(input_data);
    let r = h.finalize();
    Ok(r.as_slice()[..16].to_vec())
  } else {
    anyhow::bail!("Decryption not supported for sha1prng")
  }
}