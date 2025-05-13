/**
 * 这个函数是代码的入口
 * @param {object} config 配置信息
 * @param {object} params 参数信息
 * @returns {Promise<any>} 返回最后操作的结果
 */
async function main(config, params) {
  // 使用配置中的 'dbconn' 连接数据库
  const createTableSql = `
  CREATE TABLE test_users (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255),
    age INT,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  )
  `;
  await doAction({
    kind: 'sql',
    connection: config.dbconn,
    query: createTableSql,
  });

  // 从配置的 'datasrc' 数据文件连接读取数据
  const data = await doAction({
    kind: 'file',
    method: 'READ',
    path: config.datasrc,
  });

  // 将读取到的数据插入到 'user' 表中
  for (const row of data) {
    await doAction({
      kind: 'sql',
      connection: config.dbconn,
      query: 'INSERT INTO test_users (name, age) VALUES (?, ?)',
      rows: [[row.name, row.age]],
    });
  }

  // 查询 'age' 在 40 和 50 之间的记录
  const result = await doAction({
    kind: 'sql',
    connection: config.dbconn,
    query: 'SELECT * FROM test_users WHERE age BETWEEN 40 AND 50',
  });
  
  // 删除数据表 `test_users`
  await doAction({
    kind: 'sql',
    connection: config.dbconn,
    query: 'DROP TABLE test_users',
  });

  // 返回查询结果
  return result;
}
