use a2a_core::do_action;
use a2a_types::{Action, EMailAction, LlmAction, NotifyAction, SqlAction, Value};
use rustls::crypto::aws_lc_rs;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn setup_logging() {
  let filter = EnvFilter::from_default_env().add_directive("a2a_core=trace".parse().unwrap());
  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_level(true)
    .with_writer(std::io::stderr)
    .init();
}

#[derive(Debug, Serialize, Deserialize)]
struct NotifyConfig {
  feishu: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestConfig {
  pgsql: String,
  mysql: String,
  sqlite: String,
  email: Value,
  llm: Value,
  notify: NotifyConfig,
}

#[tokio::test]
async fn test_sql() {
  let config_data = include_str!("./config.json");

  let conf = serde_json::from_str::<TestConfig>(config_data).unwrap();

  //let connections = vec![conf.pgsql.clone(), conf.mysql.clone(), conf.sqlite.clone()];
  let connections = vec![conf.sqlite.clone()];

  let actions = vec![
    SqlAction {
      query: r#"
      CREATE TABLE IF NOT EXISTS a2a_test (
        id INT PRIMARY KEY,
        name TEXT NOT NULL,
        last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
      );
      "#
      .to_string(),
      ..Default::default()
    },
    SqlAction {
      query: r#"
      INSERT INTO a2a_test (id, name) VALUES (?, ?);
      "#
      .to_string(),
      rows: Some(json!([[1, "user1"], [2, "user2"]])),
      ..Default::default()
    },
    SqlAction {
      query: r#"
      SELECT * FROM a2a_test;
      "#
      .to_string(),
      ..Default::default()
    },
    SqlAction {
      query: r#"
      SELECT * FROM a2a_test WHERE id = ?;
      "#
      .to_string(),
      rows: Some(json!([1])),
      ..Default::default()
    },
    SqlAction {
      query: r#"
      DROP TABLE IF EXISTS a2a_test;
      "#
      .to_string(),
      ..Default::default()
    },
  ];

  for conn in connections {
    for action in actions.iter() {
      let action = action.clone();
      let action = Action::Sql(SqlAction {
        connection: conn.clone(),
        ..action
      });
      let result = do_action(action).await.unwrap();
      println!("{}", serde_json::to_string_pretty(&result).unwrap());
    }
  }
}

#[tokio::test]
async fn test_email() {
  setup_logging();
  rustls::crypto::CryptoProvider::install_default(aws_lc_rs::default_provider()).unwrap();
  let config_data = include_str!("./config.json");

  let conf = serde_json::from_str::<TestConfig>(config_data).unwrap();
  info!("config: {:?}", conf);

  let action = EMailAction {
    account: conf.email.clone(),
    method: "READ".to_string(),
    last_id: Some(218),
    ..Default::default()
  };

  match do_action(Action::EMail(action)).await {
    Ok(result) => println!("{}", serde_json::to_string_pretty(&result).unwrap()),
    Err(err) => eprintln!("{}", err),
  }
}

#[tokio::test]
async fn test_shell() {
  setup_logging();
  let action = a2a_types::ShellAction {
    command: "grep".to_string(),
    args: Some(vec![
      "=".to_string(),
      "/home/jia/repo/a2a-rs/Cargo.toml".to_string(),
    ]),
    env: None,
    cwd: None,
    override_result_mimetype: None,
  };

  match do_action(Action::Shell(action)).await {
    Ok(result) => println!("{}", serde_json::to_string_pretty(&result).unwrap()),
    Err(err) => eprintln!("{}", err),
  }
}

#[tokio::test]
async fn test_llm() {
  setup_logging();
  let config_data = include_str!("./config.json");
  let conf = serde_json::from_str::<TestConfig>(config_data).unwrap();

  let action = LlmAction {
    override_result_mimetype: Some("application/json".to_string()),
    connection: Some(conf.llm.clone()),
    user_prompt: Some("Who are you?\n reply with JSON format".to_string()),
    ..Default::default()
  };

  match do_action(Action::Llm(action)).await {
    Ok(result) => println!("{}", serde_json::to_string_pretty(&result).unwrap()),
    Err(err) => eprintln!("{}", err),
  }
}

#[tokio::test]
async fn test_notify() {
  setup_logging();
  let config_data = include_str!("./config.json");
  let conf = serde_json::from_str::<TestConfig>(config_data).unwrap();

  let action = NotifyAction {
    url: conf.notify.feishu.clone(),
    message: json!(" 量化易编 \n **Test Notification**"),
    title: Some("量化易编".to_string()),
    ..Default::default()
  };

  match do_action(Action::Notify(action)).await {
    Ok(result) => println!("{}", serde_json::to_string_pretty(&result).unwrap()),
    Err(err) => eprintln!("{}", err),
  }
}

#[tokio::test]
async fn test_enc() {
  setup_logging();
  let action = a2a_types::EncAction {
    override_result_mimetype: None,
    is_dec: Some(false),
    methods: vec!["sha1prng".to_string()],
    key: None,
    padding: None,
    data: "123456".to_string(),
  };

  match do_action(Action::Enc(action)).await {
    Ok(result) => println!("{}", serde_json::to_string_pretty(&result).unwrap()),
    Err(err) => eprintln!("{}", err),
  }
}
