/**
 * 主函数，处理数据库操作和数据处理
 * @param {Object} config - 配置对象，包含数据库连接字符串 'dbconn' 和数据文件路径 'datasrc'
 * @param {Object} params - 参数对象，当前未使用
 * @returns {any} - 查询到的年龄在 40 到 50 之间的用户记录
 */
async function main(config, params) {
  // 创建数据表
  const createTableAction = {
    kind: 'sql',
    connection: config.dbconn,
    query: 'CREATE TABLE test_users (id INT PRIMARY KEY AUTO_INCREMENT, name VARCHAR(255), age INT, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)',
  };
  await doAction(createTableAction);

  // 读取 CSV 数据文件并插入数据
  const readFileAction = {
    kind: 'file',
    method: 'READ',
    path: config.datasrc
  };
  const csvData = await doAction(readFileAction);

  const insertDataAction = {
    kind:'sql',
    connection: config.dbconn,
    query: 'INSERT INTO test_users (name, age) VALUES (?,?)',
    rows: csvData
  };
  await doAction(insertDataAction);

  // 查询年龄在 40 到 50 之间的记录
  const queryAction = {
    kind:'sql',
    connection: config.dbconn,
    query: 'SELECT * FROM test_users WHERE age BETWEEN 40 AND 50'
  };
  const queryResult = await doAction(queryAction);

  // 删除数据表
  const dropTableAction = {
    kind:'sql',
    connection: config.dbconn,
    query: 'DROP TABLE test_users'
  };
  await doAction(dropTableAction);

  return queryResult;
}
