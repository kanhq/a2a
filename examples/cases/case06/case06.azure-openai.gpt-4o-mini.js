// written by LLM provider: azure-openai model: gpt-4o-mini
async function main(config, params) {
  // 定义 LLM 抓取索引页面并获取基金详情页面的结构化数据
  const prompts = {
    [config.index]: `type FundData = {
      code: string;
      url: string;
    }`,
  };

  // 调用抓取索引页面的操作
  const crawlAction = {
    kind: "crawl",
    browser: config.browser,
    urls: [config.index],
    llm: config.llm01,
    prompts: prompts,
  };

  const fundCodes = await doAction(crawlAction); // 获取基金详情页面的代码和 URL 数组

  // 对每个基金详情页面进行抓取
  const fundInfoPromises = fundCodes.map(async fund => {
    const detailCrawlAction = {
      kind: "crawl",
      browser: config.browser,
      urls: fund.url,
      llm: config.llm01,
      prompts: {
        [fund.url]: `type FundDetails = {
          fundName: string;
          fundType: string;
          fundScale: string;
          fundManager: string;
          fundCompany: string;
          fundEstablishmentDate: string;
          fundNetValue: string;
          fundNetValueDate: string;
          fundAccumulatedNetValue: string;
          fundAccumulatedNetValueDate: string;
          fundAnnualYield: string;
          fundAnnualYieldRanking: string;
          fundAnnualAverageYield: string;
          fundAnnualAverageYieldRanking: string;
        }`,
      },
    };

    return await doAction(detailCrawlAction); // 抓取基金详情信息
  });

  const fundInfoArray = await Promise.all(fundInfoPromises); // 等待所有抓取完成

  // 将抓取的数据写入到 fund_info.csv 文件中
  const csvAction = {
    kind: "file",
    method: "WRITE",
    path: "fund_info.csv",
    body: fundInfoArray, // 直接传递数组数据
    options: {
      hasHeader: true,
      headers: [
        "fundName",
        "fundType",
        "fundScale",
        "fundManager",
        "fundCompany",
        "fundEstablishmentDate",
        "fundNetValue",
        "fundNetValueDate",
        "fundAccumulatedNetValue",
        "fundAccumulatedNetValueDate",
        "fundAnnualYield",
        "fundAnnualYieldRanking",
        "fundAnnualAverageYield",
        "fundAnnualAverageYieldRanking",
      ],
    },
  };

  return await doAction(csvAction); // 返回写入文件的结果
}
