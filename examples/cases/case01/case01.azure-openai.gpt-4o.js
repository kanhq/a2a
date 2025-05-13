async function main(config, params) {
  // 创建测试表 'test_users'
  const createTableAction = {
    kind: "sql",
    connection: config.dbconn,
    query: `
      CREATE TABLE test_users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT,
        age INTEGER,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      )
    `
  };

  await doAction(createTableAction);

  // 读取 CSV 文件中的数据
  const readFileAction = {
    kind: "file",
    method: "READ",
    path: config.datasrc
  };

  const fileResult = await doAction(readFileAction);

  // 解析 CSV 数据并准备插入
  const insertRows = fileResult.map(row => [row.name, row.age]);

  const insertDataAction = {
    kind: "sql",
    connection: config.dbconn,
    query: "INSERT INTO test_users (name, age) VALUES (?, ?)",
    rows: insertRows
  };

  await doAction(insertDataAction);

  // 查询 'age' 在 40 和 50 之间的记录
  const queryDataAction = {
    kind: "sql",
    connection: config.dbconn,
    query: "SELECT * FROM test_users WHERE age BETWEEN 40 AND 50"
  };

  const result = await doAction(queryDataAction);

  // 删除测试表 'test_users'
  const dropTableAction = {
    kind: "sql",
    connection: config.dbconn,
    query: "DROP TABLE test_users"
  };

  await doAction(dropTableAction);

  // 返回查询结果
  return result;
}
