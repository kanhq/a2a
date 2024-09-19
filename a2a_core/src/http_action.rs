use a2a_types::{HttpAction, HttpActionResult};
use anyhow::Result;

pub async fn do_action(action: HttpAction) -> Result<HttpActionResult> {
  let mut client = reqwest::Client::builder();

  if let Some(proxy) = action.proxy.as_ref() {
    client = client.proxy(reqwest::Proxy::all(proxy)?);
  }

  let override_result_mimetype = action.override_result_mimetype.clone();

  let client = client.build()?;

  let request = to_request(action)?;

  let response = client.execute(request).await?;

  to_http_action_result(response, override_result_mimetype).await
}

fn to_request(action: HttpAction) -> Result<reqwest::Request> {
  let mut builder = reqwest::Client::new().request(
    reqwest::Method::from_bytes(action.method.as_bytes()).unwrap(),
    &action.url,
  );
  if let Some(headers) = action.headers {
    for (key, value) in headers {
      builder = builder.header(key, value);
    }
  }
  if let Some(body) = action.body {
    builder = builder.body(body);
  }
  builder.build().map_err(|e| e.into())
}

async fn to_http_action_result(
  response: reqwest::Response,
  override_mimetype: Option<String>,
) -> Result<HttpActionResult> {
  let status = response.status().as_u16();

  let headers = response
    .headers()
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
    .collect();

  let mimetype = override_mimetype
    .or_else(|| {
      response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(';').next().unwrap().to_string())
    })
    .unwrap_or_default();

  let body = response.bytes().await?;

  Ok(HttpActionResult {
    status,
    headers: Some(headers),
    body: a2a_tojson::bytes_to_json(body, &mimetype, None).ok(),
  })
}
