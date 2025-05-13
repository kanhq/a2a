// written by LLM provider: lingyi model: yi-lightning
export async function main(config, params) {
  // 第一步：使用数据库连接创建 'test_users' 表
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
    `,
  };
  await doAction(createTableAction);

  // 第二步：从 CSV 文件读取数据
  const fileAction = {
    kind: "file",
    method: "READ",
    path: config.datasrc,
    options: {
      hasHeader: true, // 假设 CSV 文件有标题行
      delimiter: ",", // 假设 CSV 使用逗号作为分隔符
    },
  };
  const fileResult = await doAction(fileAction);

  // 第三步：将读取到的数据插入到 'test_users' 表中
  const insertQuery = `
    INSERT INTO test_users (name, age)
    VALUES (?, ?)
  `;
  for (const row of fileResult) {
    const insertAction = {
      kind: "sql",
      connection: config.dbconn,
      query: insertQuery,
      rows: [[row.name, row.age]], // 假设 CSV 文件的列名是 'name' 和 'age'
    };
    await doAction(insertAction);
  }

  // 第四步：查询 'age' 在 40 和 50 之间的记录
  const selectQuery = `
    SELECT * FROM test_users
    WHERE age BETWEEN 40 AND 50
  `;
  const selectAction = {
    kind: "sql",
    connection: config.dbconn,
    query: selectQuery,
  };
  const selectResult = await doAction(selectAction);

  // 第五步：删除 'test_users' 表
  const dropTableAction = {
    kind: "sql",
    connection: config.dbconn,
    query: "DROP TABLE test_users",
  };
  await doAction(dropTableAction);

  // 返回查询结果
  return selectResult;
}
