// written by LLM provider: deepseek model: deepseek-chat
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

  // 从 CSV 文件读取数据
  const readFileAction = {
    kind: "file",
    method: "READ",
    path: config.datasrc,
    options: {
      hasHeader: true,
      headers: ["name", "age"],
      delimiter: ","
    }
  };
  const fileData = await doAction(readFileAction);

  // 将读取到的数据插入到 'test_users' 表中
  const insertDataAction = {
    kind: "sql",
    connection: config.dbconn,
    query: `
      INSERT INTO test_users (name, age) VALUES ?
    `,
    rows: fileData.map(row => [row.name, row.age])
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
  const queryResult = await doAction(queryAction);

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
  return queryResult;
}
