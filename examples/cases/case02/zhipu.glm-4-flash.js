// 假设配置和参数已经被定义
// const config = ...;
// const params = ...;

// 主函数，程序的入口点
async function main() {
  // 从newsdb_provider表读取所有provider
  const providers = await doAction({
    kind: 'sql',
    connection: 'newsdb_provider',
    query: 'SELECT * FROM newsdb_provider',
  });

  // 存储所有成功的请求数据
  let allNewsData = [];

  // 并行处理每个provider
  for (const provider of providers) {
    try {
      // 发起HTTP GET请求
      const response = await doAction({
        kind: 'http',
        method: 'GET',
        url: provider.url,
      });

      // 检查状态码
      if (response.status === 0) {
        // 将返回的数据添加到数组中
        allNewsData = allNewsData.concat(response.body.data);
      }
    } catch (error) {
      // 如果请求失败，忽略该provider
      continue;
    }
  }

  // 按时间排序
  allNewsData.sort((a, b) => a.time - b.time);

  // 将所有成功的数据更新到news表中
  const newsDataForInsert = allNewsData.map(newsItem => ({
    title: newsItem.title,
    content: newsItem.content,
    time: newsItem.time,
  }));

  // 执行SQL插入操作
  await doAction({
    kind: 'sql',
    connection: 'newsdb',
    query: 'INSERT INTO news (title, content, time) VALUES ?',
    datas: [newsDataForInsert],
  });
}

// 确保main函数不会被意外调用
if (import.meta.url !== 'main.js') {
  main();
}
