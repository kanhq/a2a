async function main(config, params) {
  // 创建数据表 'test_users'
  const createTableSql = {
    kind: 'sql',
    connection: config.dbconn,
    query: `
      CREATE TABLE IF NOT EXISTS test_users (
        id INT PRIMARY KEY AUTO_INCREMENT,
        name VARCHAR(255),
        age INT,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      )
    `
  };
  await doAction(createTableSql);

  // 从配置的 'datasrc' 数据文件读取数据
  const fileReadAction = {
    kind: 'file',
    method: 'READ',
    path: config.datasrc
  };
  const fileData = await doAction(fileReadAction);

  // 将读取到的数据插入到 'test_users' 表中
  const insertSql = {
    kind: 'sql',
    connection: config.dbconn,
    query: 'INSERT INTO test_users (name, age) VALUES (?, ?)',
    rows: fileData.map(row => [row.name, row.age])
  };
  await doAction(insertSql);

  // 查询 'age' 在 40 和 50 之间的记录
  const selectSql = {
    kind: 'sql',
    connection: config.dbconn,
    query: 'SELECT * FROM test_users WHERE age BETWEEN ? AND ?',
    rows: [[40, 50]]
  };
  const result = await doAction(selectSql);

  // 删除数据表 'test_users'
  const dropTableSql = {
    kind: 'sql',
    connection: config.dbconn,
    query: 'DROP TABLE IF EXISTS test_users'
  };
  await doAction(dropTableSql);

  // 返回查询结果
  return result;
}
