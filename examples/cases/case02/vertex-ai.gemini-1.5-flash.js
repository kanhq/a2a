// 获取 provider 列表
async function getProviders() {
  const sqlAction = {
    kind: "sql",
    connection: config.newsdb_provider,
    query: "SELECT * FROM newsdb_provider",
  };
  const providers = await doAction(sqlAction);
  return providers.rows;
}

// 获取新闻数据
async function getNews(provider) {
  const httpAction = {
    kind: "http",
    method: "GET",
    url: provider.url,
  };
  const response = await doAction(httpAction);
  if (response.status === 0) {
    return response.body.data;
  }
  return [];
}

// 更新新闻数据
async function updateNews(news) {
  const sqlAction = {
    kind: "sql",
    connection: config.newsdb,
    query: "INSERT INTO news (title, content, time) VALUES (?, ?, ?)",
    datas: news.map((item) => [item.title, item.content, item.time]),
  };
  await doAction(sqlAction);
}

// 主函数
async function main() {
  const providers = await getProviders();
  const news = [];
  // 并行获取新闻数据
  for (const provider of providers) {
    const providerNews = await getNews(provider);
    news.push(...providerNews);
  }
  // 按照时间排序
  news.sort((a, b) => a.time - b.time);
  // 更新新闻数据
  await updateNews(news);
}
