/**
 * 入口函数
 * @param {object} config 应用配置
 * @param {object} params 应用参数
 * @returns {Promise<any>}
 */
async function main(config, params) {
  // 使用配置中的 'dbconn' 连接数据库
  const connection = config.dbconn;

  // 创建一个 SQL 动作来创建数据表
  const createTableAction = {
    kind: "sql",
    connection: connection,
    query: `
      CREATE TABLE IF NOT EXISTS test_users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT,
        age INTEGER,
        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
      );
    `,
  };

  // 执行创建数据表的动作
  await doAction(createTableAction);

  // 读取 'datasrc' 数据文件
  const fileAction = {
    kind: "file",
    method: "READ",
    path: config.datasrc,
  };

  // 执行读取数据文件的动作
  const data = await doAction(fileAction);
  const csvData = [];
  data.split("\n").forEach((row) => {
    const [name, age] = row.split(",");
    csvData.push([name, age]);
  });

  // 在数据表中插入读取到的数据
  const insertDataAction = {
    kind: "sql",
    connection: connection,
    query: "INSERT INTO test_users (name, age) VALUES (?, ?);",
    rows: csvData,
  };

  // 执行插入数据的动作
  await doAction(insertDataAction);

  // 查询 'age' 在 40 和 50 之间的记录
  const queryAction = {
    kind: "sql",
    connection: connection,
    query: "SELECT * FROM test_users WHERE age BETWEEN 40 AND 50;",
  };

  // 执行查询动作
  const result = await doAction(queryAction);

  // 删除数据表
  const dropTableAction = {
    kind: "sql",
    connection: connection,
    query: "DROP TABLE test_users;",
  };

  // 执行删除数据表的动作
  await doAction(dropTableAction);

  // 返回最终的结果
  return result;
}
