async function main(config, params) {
  const sqlAction = {
    kind: "sql",
    connection: config.dbconn,
    query: "CREATE TABLE IF NOT EXISTS test_users (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(255), age INT, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)",
  };
  await doAction(sqlAction);

  const fileAction = {
    kind: "file",
    path: config.datasrc,
    method: "READ",
  };
  const fileResult = await doAction(fileAction);
  const csvData = fileResult;

  const sqlInsert = "INSERT INTO test_users (name, age) VALUES (?, ?)";
  const insertActions = csvData.map((row) => {
    return {
      kind: "sql",
      connection: config.dbconn,
      query: sqlInsert,
      rows: [row.split(",")[0], parseInt(row.split(",")[1])],
    };
  });

  for (const action of insertActions) {
    await doAction(action);
  }

  const sqlQuery = {
    kind: "sql",
    connection: config.dbconn,
    query: "SELECT * FROM test_users WHERE age BETWEEN 40 AND 50",
  };
  const result = await doAction(sqlQuery);
  return result;
}
