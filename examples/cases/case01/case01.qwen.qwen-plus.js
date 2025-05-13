// 定义主函数，接收配置和参数
async function main(config, params) {
  // 创建数据表的 SQL 语句
  const createTableSql = {
    kind: "sql",
    connection: config.dbconn,
    query: "CREATE TABLE test_users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, age INTEGER, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)",
  };

  // 执行创建数据表的操作
  await doAction(createTableSql);

  // 读取 CSV 文件
  const readFileAction = {
    kind: "file",
    method: "READ",
    path: config.datasrc,
    overrideResultMimeType: "text/csv"
  };

  // 读取 CSV 文件内容并解析为数组
  const csvData = await doAction(readFileAction);

  // 插入数据的 SQL 语句
  const insertSql = {
    kind: "sql",
    connection: config.dbconn,
    query: "INSERT INTO test_users (name, age) VALUES (?, ?)",
    rows: csvData.map(row => [row.name, parseInt(row.age)])
  };

  // 执行插入操作
  await doAction(insertSql);

  // 查询年龄在 40 到 50 之间的用户
  const selectSql = {
    kind: "sql",
    connection: config.dbconn,
    query: "SELECT * FROM test_users WHERE age BETWEEN ? AND ?",
    rows: [[40, 50]]
  };

  // 执行查询操作
  const queryResult = await doAction(selectSql);

  // 删除数据表
  const dropTableSql = {
    kind: "sql",
    connection: config.dbconn,
    query: "DROP TABLE test_users"
  };

  // 执行删除数据表的操作
  await doAction(dropTableSql);

  // 返回查询结果
  return queryResult;
}
