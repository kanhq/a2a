// 定义主函数，接收配置和参数
async function main(config, params) {
  // 创建数据表的SQL语句
  const createTableSql = {
    kind: "sql",
    connection: config.dbconn,
    query: "CREATE TABLE test_users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT, age INTEGER, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)",
  };

  // 执行创建数据表的操作
  await doAction(createTableSql);

  // 读取CSV文件内容
  const readFileAction = {
    kind: "file",
    method: "READ",
    path: config.datasrc,
    overrideResultMimeType: "text/csv"
  };

  // 将读取到的数据插入到用户表中
  const insertDataSql = {
    kind: "sql",
    connection: config.dbconn,
    query: "INSERT INTO test_users (name, age) VALUES (?, ?)",
    rows: await doAction(readFileAction)
  };

  // 执行插入操作
  await doAction(insertDataSql);

  // 查询年龄在40到50之间的记录
  const selectSql = {
    kind: "sql",
    connection: config.dbconn,
    query: "SELECT * FROM test_users WHERE age BETWEEN ? AND ?",
    rows: [[40, 50]]
  };

  // 执行查询操作并获取结果
  const result = await doAction(selectSql);

  // 删除数据表
  const dropTableSql = {
    kind: "sql",
    connection: config.dbconn,
    query: "DROP TABLE test_users"
  };

  // 执行删除表操作
  await doAction(dropTableSql);

  // 返回查询结果
  return result;
}
