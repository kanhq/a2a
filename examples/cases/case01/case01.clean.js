
await doAction({
  kind: "sql",
  connection: config.dbconn,
  query: "DROP TABLE test_users"
});