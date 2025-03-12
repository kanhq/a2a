use aes::cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit};
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use aes::Aes128;
use anyhow::{anyhow, bail, Result};
use rand::RngCore;

pub(crate) fn padding_as(method: &str, block_size: usize, data: &[u8]) -> Vec<u8> {
  match method {
    "zero" => {
      if data.len() % block_size == 0 {
        data.to_vec()
      } else {
        let mut out = data.to_vec();
        out.resize(data.len() + block_size - data.len() % block_size, 0);
        out
      }
    }
    "space" => {
      if data.len() % block_size == 0 {
        data.to_vec()
      } else {
        let mut out = data.to_vec();
        out.resize(data.len() + block_size - data.len() % block_size, 20);
        out
      }
    }
    "pkcs5" | "pcks7" => {
      let padding = block_size - data.len() % block_size;
      let mut out = data.to_vec();
      out.resize(data.len() + padding, padding as u8);
      out
    }
    _ => data.to_vec(),
  }
}

pub(crate) fn unpadding_as(method: &str, data: &[u8]) -> Vec<u8> {
  match method {
    "pkcs5" | "pcks7" => {
      let padding = data.last().unwrap_or(&0);
      let padding = *padding as usize;
      let mut out = data.to_vec();
      if padding > 0 && padding <= out.len() {
        let padding = out.len() - padding;
        out.resize(padding, 0);
      }
      out
    }
    _ => data.to_vec(),
  }
}

pub(crate) fn aes_ecb_action(
  is_enc: bool,
  input_data: &[u8],
  key: Option<&[u8]>,
  padding: &str,
) -> Result<Vec<u8>> {
  const BLOCK_SIZE: usize = 16;
  if key.map(|k| k.len()).unwrap_or_default() != BLOCK_SIZE {
    bail!("key is required and have 16 bytes length")
  }
  let key = key.unwrap();
  let cipher = Aes128::new(GenericArray::from_slice(key));

  if is_enc {
    let mut out = padding_as(padding, BLOCK_SIZE, input_data);
    if out.len() % BLOCK_SIZE != 0 {
      bail!("Invalid padding")
    }
    for chunk in out.chunks_exact_mut(BLOCK_SIZE) {
      cipher.encrypt_block(&mut GenericArray::from_mut_slice(chunk));
    }
    Ok(out)
  } else {
    let mut out = unpadding_as(padding, input_data);
    if out.len() % BLOCK_SIZE != 0 || out.len() < BLOCK_SIZE {
      bail!("Invalid padding")
    }
    for chunk in out.chunks_exact_mut(BLOCK_SIZE) {
      cipher.decrypt_block(&mut GenericArray::from_mut_slice(chunk));
    }
    Ok(out)
  }
}

pub(crate) fn aes_cbc_action(
  is_enc: bool,
  input_data: &[u8],
  key: Option<&[u8]>,
  padding: &str,
) -> Result<Vec<u8>> {
  type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
  type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
  const BLOCK_SIZE: usize = 16;

  if key.map(|k| k.len()).unwrap_or_default() != BLOCK_SIZE {
    bail!("key is required and have 16 bytes length")
  }
  let key = key.unwrap();

  if is_enc {
    let mut out = padding_as(padding, BLOCK_SIZE, input_data);
    if out.len() % BLOCK_SIZE != 0 {
      bail!("Invalid padding")
    }
    let mut iv = [0u8; BLOCK_SIZE];
    rand::rng().fill_bytes(&mut iv);
    let cipher = Aes128CbcEnc::new_from_slices(key, &iv).unwrap();
    let msg_len = out.len();
    cipher
      .encrypt_padded_mut::<aes::cipher::block_padding::NoPadding>(out.as_mut_slice(), msg_len)
      .map_err(|e| anyhow!("{}", e))?;
    out.extend_from_slice(&iv);
    Ok(out)
  } else {
    let mut out = unpadding_as(padding, input_data);
    if out.len() % BLOCK_SIZE != 0 || out.len() < BLOCK_SIZE {
      bail!("Invalid padding")
    }
    let iv = out.split_off(out.len() - BLOCK_SIZE);
    let cipher = Aes128CbcDec::new_from_slices(key, &iv).unwrap();
    cipher
      .decrypt_padded_mut::<aes::cipher::block_padding::NoPadding>(&mut out)
      .map_err(|e| anyhow!("{}", e))?;
    Ok(out)
  }
}
