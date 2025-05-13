async function main() {
  // 从配置中获取数据库连接字符串
  const newsdbProviderConnection = config.newsdb_provider;
  const newsdbConnection = config.newsdb;

  // 定义 SQL 查询以获取所有的 provider
  const getProvidersQuery = "SELECT name, url, last_fetch_time FROM newsdb_provider";

  // 执行 SQL 查询
  const providersResult = await doAction({
    kind: "sql",
    connection: newsdbProviderConnection,
    query: getProvidersQuery
  });

  // 获取所有的 provider
  const providers = providersResult.rows;

  // 并行发起 HTTP GET 请求
  const fetchPromises = providers.map(async (provider) => {
    const { url, last_fetch_time } = provider;
    const response = await doAction({
      kind: "http",
      method: "GET",
      url: `${url}?last_fetch_time=${last_fetch_time}`
    });

    // 检查状态码
    if (response.status === 0) {
      return response.body.data;
    } else {
      return [];
    }
  });

  // 等待所有请求完成
  const fetchResults = await Promise.all(fetchPromises);

  // 合并所有成功的请求数据
  const mergedData = fetchResults.flat();

  // 按照 time 排序
  mergedData.sort((a, b) => a.time - b.time);

  // 定义 SQL 插入语句
  const insertNewsQuery = "INSERT INTO news (title, content, time) VALUES (?, ?, ?)";

  // 并行插入所有新闻数据
  const insertPromises = mergedData.map(async (news) => {
    await doAction({
      kind: "sql",
      connection: newsdbConnection,
      query: insertNewsQuery,
      rows: [news.title, news.content, news.time]
    });
  });

  // 等待所有插入操作完成
  await Promise.all(insertPromises);
}

// 调用主函数
main();
