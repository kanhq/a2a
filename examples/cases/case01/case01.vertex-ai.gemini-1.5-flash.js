async function main(config, params) {
  // 使用配置中的 'dbconn' 连接数据库
  const dbconn = config.dbconn;
  // 创建数据表 'test_users'
  await doAction({
    kind: "sql",
    connection: dbconn,
    query: `CREATE TABLE test_users (
      id INT AUTO_INCREMENT PRIMARY KEY,
      name VARCHAR(255),
      age INT,
      updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )`
  });

  // 从配置的 'datasrc' 数据文件连接读取数据
  const datasrc = config.datasrc;
  const data = await doAction({
    kind: "file",
    method: "READ",
    path: datasrc
  });

  // 将读取到的数据插入到 'user' 表中
  const rows = data.map(row => [row.name, row.age]);
  await doAction({
    kind: "sql",
    connection: dbconn,
    query: "INSERT INTO test_users (name, age) VALUES (?, ?)",
    rows: rows
  });

  // 在数据表中查询 'age' 在 40 和 50 之间的记录
  const result = await doAction({
    kind: "sql",
    connection: dbconn,
    query: "SELECT * FROM test_users WHERE age BETWEEN 40 AND 50"
  });

  // 删除掉数据表 `test_users`
  await doAction({
    kind: "sql",
    connection: dbconn,
    query: "DROP TABLE test_users"
  });

  return result;
}
