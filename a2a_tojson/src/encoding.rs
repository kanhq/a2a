use anyhow::Result;
use encoding_rs::{Encoding, BIG5, EUC_JP, EUC_KR, GB18030, UTF_16BE, UTF_16LE, WINDOWS_1252};

static ALL_ENCODINGS: &[&Encoding] = &[
  GB18030,
  WINDOWS_1252,
  BIG5,
  UTF_16BE,
  UTF_16LE,
  EUC_JP,
  EUC_KR,
];

pub fn try_to_utf8(input: Vec<u8>) -> Result<String> {
  match String::from_utf8(input) {
    // If the input is valid UTF-8, return it directly
    Ok(s) => return Ok(s),
    Err(e) => {
      // get input back as bytes
      let input = e.into_bytes();

      // If that fails, try all known encodings
      for &encoding in ALL_ENCODINGS {
        let (encoding, without_bom) = match Encoding::for_bom(input.as_slice()) {
          Some((encoding, bom_length)) => (encoding, &input[bom_length..]),
          None => (encoding, input.as_slice()),
        };
        if let Some(cow) = encoding.decode_without_bom_handling_and_without_replacement(without_bom)
        {
          return Ok(cow.into_owned());
        }
      }
      return Err(anyhow::anyhow!("Failed to decode input"));
    }
  }
}
