// 定义一个异步主函数
async function main() {
  // 从配置中获取数据库连接信息
  const dbConnection = config.newsdb_provider;

  // 构建查询 provider 的 SQL 语句
  const queryProviders = `SELECT name, url, last_fetch_time FROM ${dbConnection}`;

  // 执行 SQL 查询获取所有 provider
  const providers = await doAction({
    kind: 'sql',
    connection: config.dbConnection,
    query: queryProviders
  });

  // 并行发起 HTTP 请求
  const httpRequests = providers.rows.map(provider => {
    return doAction({
      kind: 'http',
      method: 'GET',
      url: provider.url
    });
  });

  // 等待所有 HTTP 请求完成
  const results = await Promise.all(httpRequests);

  // 过滤出状态码为 0 的请求结果
  const successfulResults = results.filter(result => result.status === 0);

  // 合并所有成功请求的数据
  let combinedData = [];
  successfulResults.forEach(result => {
    combinedData = combinedData.concat(result.body.data);
  });

  // 按照 time 排序
  combinedData.sort((a, b) => a.time - b.time);

  // 构建 SQL 插入语句
  const insertQuery = `INSERT INTO news (title, content, time) VALUES ?`;

  // 准备插入数据
  const insertData = combinedData.map(item => [item.title, item.content, item.time]);

  // 执行 SQL 插入操作
  await doAction({
    kind: 'sql',
    connection: config.dbConnection,
    query: insertQuery,
    datas: [insertData]
  });
}

// 注意：main 函数不应在此脚本中被调用
