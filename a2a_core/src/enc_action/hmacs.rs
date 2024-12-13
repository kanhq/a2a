
use anyhow::Result;
use hmac::{Hmac, Mac};


pub(crate) fn hmac_md5_action(is_enc: bool, input_data: &[u8], key: Option<&[u8]>, _padding: &str ) -> Result<Vec<u8>> {
  if key.is_none() {
    anyhow::bail!("Key is required for hmac_md5")
  }
  let key = key.unwrap();
  if is_enc {
    let mut mac = Hmac::<md5::Md5>::new_from_slice(key)?;
    mac.update(input_data);
    let r= mac.finalize();
    Ok(r.into_bytes().to_vec())
  } else {
    anyhow::bail!("Decryption not supported for hmac_md5")
  }
}

pub(crate) fn hmac_sha1_action(is_enc: bool, input_data: &[u8], key: Option<&[u8]>, _padding: &str ) -> Result<Vec<u8>> {
  if key.is_none() {
    anyhow::bail!("Key is required for hmac_sha1")
  }
  let key = key.unwrap();
  if is_enc {
    let mut mac = Hmac::<sha1::Sha1>::new_from_slice(key)?;
    mac.update(input_data);
    let r= mac.finalize();
    Ok(r.into_bytes().to_vec())
  } else {
    anyhow::bail!("Decryption not supported for hmac_sha1")
  }
}

pub(crate) fn hmac_sha256_action(is_enc: bool, input_data: &[u8], key: Option<&[u8]>, _padding: &str ) -> Result<Vec<u8>> {
  if key.is_none() {
    anyhow::bail!("Key is required for hmac_sha256")
  }
  let key = key.unwrap();
  if is_enc {
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(key)?;
    mac.update(input_data);
    let r= mac.finalize();
    Ok(r.into_bytes().to_vec())
  } else {
    anyhow::bail!("Decryption not supported for hmac_sha256")
  }
}