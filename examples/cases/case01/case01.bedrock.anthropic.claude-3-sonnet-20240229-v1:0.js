// 定义入口函数
async function main(config, params) {
  // 从配置获取数据库连接字符串
  const dbConnection = config.dbconn;

  // 创建数据表
  const createTableQuery = `
    CREATE TABLE test_users (
      id INT AUTO_INCREMENT PRIMARY KEY,
      name VARCHAR(255),
      age INT,
      updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )
  `;
  await doAction({ kind: 'sql', connection: dbConnection, query: createTableQuery });

  // 读取数据文件
  const dataFile = config.datasrc;
  const fileData = await doAction({ kind: 'file', method: 'READ', path: dataFile });

  // 将数据插入表中
  const insertQuery = 'INSERT INTO test_users (name, age) VALUES ?';
  const rows = fileData.map(row => [row.name, row.age]);
  await doAction({ kind: 'sql', connection: dbConnection, query: insertQuery, rows });

  // 查询符合条件的记录
  const selectQuery = 'SELECT * FROM test_users WHERE age BETWEEN 40 AND 50';
  const result = await doAction({ kind: 'sql', connection: dbConnection, query: selectQuery });

  // 删除数据表
  const dropTableQuery = 'DROP TABLE test_users';
  await doAction({ kind: 'sql', connection: dbConnection, query: dropTableQuery });

  return result;
}
