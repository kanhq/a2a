async function main(config, params) {
  // 创建test_users表
  const createTableSql = {
    kind: "sql",
    connection: config.dbconn,
    query: `
      CREATE TABLE IF NOT EXISTS test_users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT,
        age INTEGER,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      )
    `
  };
  await doAction(createTableSql);

  // 读取CSV文件
  const readCsvFile = {
    kind: "file",
    method: "READ",
    path: config.datasrc
  };
  const csvData = await doAction(readCsvFile);

  // 插入数据到test_users表
  const insertDataSql = {
    kind: "sql",
    connection: config.dbconn,
    query: "INSERT INTO test_users (name, age) VALUES (?, ?)",
    rows: csvData.map(row => [row.name, row.age])
  };
  await doAction(insertDataSql);

  // 查询age在40到50之间的记录
  const queryDataSql = {
    kind: "sql",
    connection: config.dbconn,
    query: "SELECT * FROM test_users WHERE age BETWEEN 40 AND 50"
  };
  const queryResult = await doAction(queryDataSql);

  // 删除test_users表
  const dropTableSql = {
    kind: "sql",
    connection: config.dbconn,
    query: "DROP TABLE IF EXISTS test_users"
  };
  await doAction(dropTableSql);

  // 返回查询结果
  return queryResult;
}
