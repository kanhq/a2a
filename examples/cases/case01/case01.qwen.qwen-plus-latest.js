export async function main(config, params) {
  // 创建数据表
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

  // 读取 CSV 文件
  const readCsvAction = {
    kind: "file",
    method: "READ",
    path: config.datasrc,
    overrideResultMimeType: "text/csv"
  };
  const csvData = await doAction(readCsvAction);

  // 插入数据到表中
  const insertDataAction = {
    kind: "sql",
    connection: config.dbconn,
    query: `INSERT INTO test_users (name, age) VALUES (?, ?)`,
    rows: csvData.map(row => [row.name, row.age])
  };
  await doAction(insertDataAction);

  // 查询年龄在 40 到 50 之间的记录
  const selectDataAction = {
    kind: "sql",
    connection: config.dbconn,
    query: `SELECT * FROM test_users WHERE age BETWEEN 40 AND 50`
  };
  const result = await doAction(selectDataAction);

  // 删除数据表
  const dropTableAction = {
    kind: "sql",
    connection: config.dbconn,
    query: `DROP TABLE test_users`
  };
  await doAction(dropTableAction);

  // 返回查询结果
  return result;
}
