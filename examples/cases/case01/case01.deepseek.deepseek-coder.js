export async function main(config, params) {
  // 创建数据表 'test_users'
  const createTableAction = {
    kind: "sql",
    connection: config.dbconn,
    query: `
      CREATE TABLE test_users (
        id INT PRIMARY KEY AUTO_INCREMENT,
        name VARCHAR(255),
        age INT,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      )
    `
  };
  await doAction(createTableAction);

  // 从 'datasrc' 读取 CSV 数据
  const readFileAction = {
    kind: "file",
    method: "READ",
    path: config.datasrc
  };
  const csvData = await doAction(readFileAction);

  // 将 CSV 数据插入到 'test_users' 表中
  const insertDataAction = {
    kind: "sql",
    connection: config.dbconn,
    query: `
      INSERT INTO test_users (name, age) VALUES ?
    `,
    rows: csvData.map(row => [row.name, row.age])
  };
  await doAction(insertDataAction);

  // 查询 'age' 在 40 和 50 之间的记录
  const queryAction = {
    kind: "sql",
    connection: config.dbconn,
    query: `
      SELECT * FROM test_users WHERE age BETWEEN 40 AND 50
    `
  };
  const result = await doAction(queryAction);

  // 删除数据表 'test_users'
  const dropTableAction = {
    kind: "sql",
    connection: config.dbconn,
    query: `
      DROP TABLE test_users
    `
  };
  await doAction(dropTableAction);

  // 返回查询结果
  return result;
}
