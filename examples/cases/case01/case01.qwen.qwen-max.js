async function main(config, params) {
  // 创建数据表 test_users
  const createTableSql = `
    CREATE TABLE IF NOT EXISTS test_users (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT,
      age INTEGER,
      updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )
  `;
  await doAction({
    kind: "sql",
    connection: config.dbconn,
    query: createTableSql,
    rows: []
  });

  // 读取 CSV 文件数据
  const csvData = await doAction({
    kind: "file",
    method: "READ",
    path: config.datasrc
  });

  // 插入数据到 test_users 表
  const insertPromises = csvData.map(row => {
    const insertSql = "INSERT INTO test_users (name, age) VALUES (?, ?)";
    return doAction({
      kind: "sql",
      connection: config.dbconn,
      query: insertSql,
      rows: [[row.name, row.age]]
    });
  });
  await Promise.all(insertPromises);

  // 查询 age 在 40 和 50 之间的记录
  const querySql = "SELECT * FROM test_users WHERE age BETWEEN 40 AND 50";
  const queryResult = await doAction({
    kind: "sql",
    connection: config.dbconn,
    query: querySql,
    rows: []
  });

  // 删除数据表 test_users
  const dropTableSql = "DROP TABLE test_users";
  await doAction({
    kind: "sql",
    connection: config.dbconn,
    query: dropTableSql,
    rows: []
  });

  // 返回查询结果
  return queryResult;
}
