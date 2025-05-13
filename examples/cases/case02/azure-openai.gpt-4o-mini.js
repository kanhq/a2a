// 定义主函数，作为代码的入口
async function main() {
  // 从配置中读取数据库连接
  const connection = config.newsdb; // 假设 config 中包含 database 连接的配置

  // 1. 从 newsdb_provider 表中读取所有的 provider
  const providerQuery = `
    SELECT name, url, last_fetch_time FROM newsdb_provider;
  `;

  const providersResult = await doAction({
    kind: 'sql',
    connection: connection,
    query: providerQuery
  });

  // 提取提供者信息
  const providers = providersResult.rows;

  // 2. 并行发送 HTTP GET 请求给每个 provider 的 URL
  const fetchPromises = providers.map(provider => {
    return fetch(provider.url)
      .then(response => response.json())
      .then(data => {
        // 检查状态码
        if (data.status === 0) {
          // 如果状态码是 0，则返回数据
          return data.data.map(item => ({
            title: item.title,
            content: item.content,
            time: item.time
          }));
        }
        return []; // 否则返回空数组
      });
  });

  // 等待所有请求完成，并将结果合并
  const results = await Promise.all(fetchPromises);
  const mergedData = results.flat(); // 合并所有成功的结果

  // 3. 按照 time 对数据进行排序
  const sortedData = mergedData.sort((a, b) => a.time - b.time);

  // 4. 更新到 news 表中
  const insertPromises = sortedData.map(item => {
    const insertQuery = `
      INSERT INTO news (title, content, time) VALUES (?, ?, ?);
    `;

    return doAction({
      kind: 'sql',
      connection: connection,
      query: insertQuery,
      datas: [item.title, item.content, item.time]
    });
  });

  // 等待所有插入操作完成
  await Promise.all(insertPromises);
}
