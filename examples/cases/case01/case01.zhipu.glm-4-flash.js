async function main(config, params) {
  // 创建一个 SQL Action，用于创建数据表 'test_users'
  const createTableAction = {
    kind: 'sql',
    connection: config.dbconn,
    query: `
      CREATE TABLE IF NOT EXISTS test_users (
        id INT PRIMARY KEY AUTO_INCREMENT,
        name VARCHAR(255),
        age INT,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );
    `,
  };

  // 执行创建表的 SQL Action
  await doAction(createTableAction);

  // 读取 CSV 文件数据
  const fileReadAction = {
    kind: 'file',
    method: 'READ',
    path: config.datasrc,
  };

  // 执行文件读取 Action，并解析返回的 CSV 数据
  const fileResult = await doAction(fileReadAction);
  const csvData = fileResult.body.split('\n').map(row => row.split(','));

  // 创建一个 SQL Action，用于插入数据到 'test_users' 表中
  const insertDataAction = {
    kind: 'sql',
    connection: config.dbconn,
    query: 'INSERT INTO test_users (name, age) VALUES (?, ?);',
    rows: csvData.map(row => [row[0], parseInt(row[1], 10)]),
  };

  // 执行插入数据的 SQL Action
  await doAction(insertDataAction);

  // 创建一个 SQL Action，用于查询 'age' 在 40 和 50 之间的记录
  const queryDataAction = {
    kind: 'sql',
    connection: config.dbconn,
    query: 'SELECT * FROM test_users WHERE age BETWEEN 40 AND 50;',
  };

  // 执行查询数据的 SQL Action
  const queryResult = await doAction(queryDataAction);

  // 返回查询结果
  return queryResult;
}

// 假设配置和参数如下所示
const config = {
  dbconn: 'your_database_connection_string',
  datasrc: 'path_to_your_csv_file.csv',
};

const params = {};

// 调用 main 函数
main(config, params).then(result => {
  // 处理结果
  console.log(result);
}).catch(error => {
  // 处理错误
  console.error(error);
});
