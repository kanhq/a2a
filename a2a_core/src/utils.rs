pub fn uuid_v7_raw() -> [u8; 16] {
  let ts = time::OffsetDateTime::now_utc().unix_timestamp_nanos() as u64 / 1_000_000;

  let mut buf = [0u8; 16];

  let mut rand_a: u16 = rand::random();
  let rand_b: u64 = rand::random();
  rand_a = 7 << 12 | rand_a >> 4;

  buf[0..8].copy_from_slice(&(ts << 16).to_be_bytes());
  buf[6..8].copy_from_slice(&rand_a.to_be_bytes());
  buf[8..16].copy_from_slice(&rand_b.to_be_bytes());
  buf
}

pub fn uuid_v7() -> String {
  let raw = uuid_v7_raw();
  let mut raw_hex = [0u8; 32];

  let _ = hex_simd::encode(
    &raw,
    hex_simd::Out::from_slice(&mut raw_hex),
    hex_simd::AsciiCase::Lower,
  );

  let mut uuid = String::with_capacity(36);
  unsafe {
    uuid.push_str(std::str::from_utf8_unchecked(&raw_hex[0..8]));
    uuid.push('-');
    uuid.push_str(std::str::from_utf8_unchecked(&raw_hex[8..12]));
    uuid.push('-');
    uuid.push_str(std::str::from_utf8_unchecked(&raw_hex[12..16]));
    uuid.push('-');
    uuid.push_str(std::str::from_utf8_unchecked(&raw_hex[16..20]));
    uuid.push('-');
    uuid.push_str(std::str::from_utf8_unchecked(&raw_hex[20..32]));
  }

  uuid
}
