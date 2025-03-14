use std::{sync::Arc, time::Duration};

use a2a_types::{WebSearchAction, WebSearchActionResult, WebSearchResult};
use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions, Tab};
use reqwest::Url;
use tracing::{debug, warn};

use super::SerializableLaunchOptions;

struct SearchProvider {
  name: String,
  url: String,
  link_selector: String,
}

pub async fn do_action(action: WebSearchAction) -> Result<WebSearchActionResult> {
  let mut launch_options = action
    .browser
    .clone()
    .and_then(|browser| serde_json::from_value::<SerializableLaunchOptions>(browser).ok())
    .unwrap_or_default();

  launch_options.setup();

  let launch_options: LaunchOptions = (&launch_options).into();

  let browser = Browser::new(launch_options)?;

  let provider = SearchProvider::from(&action);

  let pages = action.pages.unwrap_or(3) as usize;

  //let tab = browser.new_tab()?;
  //tab.navigate_to("https://www.baidu.com")?;

  match provider.search(browser, pages).await {
    Ok(pages) => {
      debug!("search result: {:#?}", pages);
      Ok(pages)
    }
    Err(err) => {
      warn!(?err, "search error");
      Err(err)
    }
  }
}

fn site_name_of_url(u: &str) -> String {
  let url = Url::parse(u).unwrap();
  let host = url.host_str().unwrap_or_default();
  let parts = host.split('.').collect::<Vec<_>>();
  if parts.len() < 3 {
    parts[0].to_string()
  } else {
    parts[1].to_string()
  }
}

impl From<&WebSearchAction> for SearchProvider {
  fn from(action: &WebSearchAction) -> Self {
    match action.provider.as_str() {
      "bing" => SearchProvider {
        name: "bing".to_string(),
        url: format!("https://cn.bing.com/search?q={}", action.query),
        link_selector: "h2 a".to_string(),
      },
      "google" => SearchProvider {
        name: "google".to_string(),
        url: format!("https://www.google.com/search?q={}", action.query),
        link_selector: "a:has(h3)".to_string(),
      },
      "baidu" => SearchProvider {
        name: "baidu".to_string(),
        url: format!("https://www.baidu.com/s?wd={}", action.query),
        link_selector: ".result h3 a".to_string(),
      },
      url => SearchProvider {
        name: site_name_of_url(url),
        url: url.replace("${query}", action.query.as_str()),
        link_selector: "a".to_string(),
      },
    }
  }
}

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36";

impl SearchProvider {
  pub async fn search(&self, browser: Browser, pages: usize) -> Result<Vec<WebSearchResult>> {
    let tab = browser.new_tab()?;
    tab.set_user_agent(USER_AGENT, None, None)?;
    tab.navigate_to(&self.url)?;
    tab.wait_until_navigated()?;
    let page_tabs = tab
      .find_elements(&self.link_selector)?
      .iter()
      .take(pages)
      .map_while(|el| {
        if let Ok(Some(href)) = el.get_attribute_value("href") {
          let tab = browser.new_tab();
          if let Ok(tab) = tab {
            tab.set_user_agent(USER_AGENT, None, None).ok();
            tab.set_default_timeout(Duration::from_secs(5));
            tab.navigate_to(&href).ok();
            return Some(tab);
          } else {
            warn!(?href, "new tab error");
          }
        }
        None
      })
      .collect::<Vec<_>>();

    let mut sets = tokio::task::JoinSet::new();

    for tab in page_tabs.into_iter() {
      sets.spawn(visit_page(tab.clone()));
    }

    let results = sets
      .join_all()
      .await
      .into_iter()
      .map_while(|res| match res {
        Ok(r) => Some(r),
        Err(err) => {
          warn!(?err, "join error");
          None
        }
      })
      .collect::<Vec<_>>();

    if results.is_empty() {
      Err(anyhow::anyhow!("no result found"))
    } else {
      Ok(results)
    }
  }
}

pub async fn visit_page(tab: Arc<Tab>) -> Result<WebSearchResult> {
  tab.wait_for_element("body")?;

  let title = tab.get_title()?;
  let url = tab.get_url();
  let icon = match tab.find_element("link[rel='icon']") {
    Ok(el) => match el.get_attribute_value("href") {
      Ok(Some(icon)) => {
        if icon.starts_with("http") {
          icon
        } else {
          match Url::parse(&url) {
            Ok(url) => format!(
              "{}://{}{}",
              url.scheme(),
              url.host_str().unwrap_or(""),
              icon
            ),
            Err(_) => String::default(),
          }
        }
      }
      _ => String::default(),
    },
    Err(_) => String::default(),
  };

  let selectors = &["article", "main", "body"];

  for selector in selectors {
    if let Ok(body) = tab
      .find_element(selector)
      .and_then(|el| el.get_inner_text())
    {
      return Ok(WebSearchResult {
        url,
        icon,
        title,
        body,
      });
    }
  }
  Err(anyhow::anyhow!("no content found"))
}
