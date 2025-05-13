async function main() {
  // 从配置中获取 `newsdb_provider` 表的连接字符串和 SQL 查询
  const connectionString = config.newsdb_provider;

  // 定义 SQL 查询语句，获取所有的 provider
  const query = "SELECT id, name, url, last_fetch_time FROM newsdb_provider";

  // 执行 SQL 查询，获取提供者列表
  const providersAction = {
    kind: "sql",
    connection: connectionString,
    query: query
  };

  const providersResult = await doAction(providersAction);
  const providers = providersResult.rows;

  // 发起并行 HTTP GET 请求
  const requests = providers.map(provider => {
    const url = `${provider.url}?last_fetch=${provider.last_fetch_time}`;
    const httpAction = {
      kind: "http",
      method: "GET",
      url: url,
    };
    return doAction(httpAction);
  });

  // 解析所有并行请求的结果
  const results = await Promise.all(requests);

  // 过滤并合并所有成功的请求数据
  const articles = [];
  results.forEach(result => {
    if (result.body.status === 0) {
      articles.push(...result.body.data);
    }
  });

  // 根据发布时间排序数据
  articles.sort((a, b) => b.time - a.time);

  // 插入新闻数据到 `news` 表中
  const insertQuery = "INSERT INTO news (title, content, time) VALUES ?";
  const insertData = articles.map(article => [article.title, article.content, article.time]);

  const newsAction = {
    kind: "sql",
    connection: config.newsdb,
    query: insertQuery,
    datas: insertData
  };

  await doAction(newsAction);
}
