use a2a_types::Value;
use glob::Pattern;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum CrawlUrl {
  Text(String),
  WithSelector {
    url: String,
    selector: Option<String>,
    wait: Option<String>,
  },
  Empty,
}

impl CrawlUrl {
  pub fn url(&self) -> &str {
    match self {
      CrawlUrl::Text(t) => t.as_str(),
      CrawlUrl::WithSelector {
        url,
        selector: _,
        wait: _,
      } => url.as_str(),
      CrawlUrl::Empty => "",
    }
  }

  pub fn selector(&self) -> &str {
    match self {
      CrawlUrl::WithSelector {
        url: _,
        selector,
        wait: _,
      } => selector.as_ref().map(|s| s.as_str()).unwrap_or("body"),
      _ => "body",
    }
  }

  pub fn wait(&self) -> &str {
    match self {
      CrawlUrl::WithSelector {
        url: _,
        selector: _,
        wait,
      } => wait.as_ref().map(|s| s.as_str()).unwrap_or(""),
      _ => "",
    }
  }
}

impl AsRef<str> for CrawlUrl {
  fn as_ref(&self) -> &str {
    self.url()
  }
}

impl From<Value> for CrawlUrl {
  fn from(value: Value) -> Self {
    serde_json::from_value(value).unwrap_or(CrawlUrl::Empty)
  }
}

#[derive(Debug, Clone)]
pub(crate) enum UrlPattern {
  Regex(Regex),
  Glob(Pattern),
  Plain(String),
}

impl UrlPattern {
  pub fn is_match(&self, url: &str) -> bool {
    match self {
      UrlPattern::Regex(re) => re.is_match(url),
      UrlPattern::Glob(g) => g.matches_with(url, glob::MatchOptions::new()),
      UrlPattern::Plain(p) => url.eq_ignore_ascii_case(p),
    }
  }
}

impl From<&str> for UrlPattern {
  fn from(s: &str) -> Self {
    if s.starts_with('/') {
      match Regex::new(&s[1..]) {
        Ok(re) => UrlPattern::Regex(re),
        Err(_) => UrlPattern::Plain(s.to_string()),
      }
    } else {
      if s.is_empty() {
        UrlPattern::Glob(Pattern::new("*").unwrap())
      } else {
        match Pattern::new(s) {
          Ok(g) => UrlPattern::Glob(g),
          Err(_) => UrlPattern::Plain(s.to_string()),
        }
      }
    }
  }
}
