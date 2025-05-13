// written by LLM provider: vertex-ai model: gemini-2.0-flash-exp
/**
 * 主函数，用于抓取基金信息并保存到文件中。
 * @param {object} config - 包含配置信息的对象。
 * @param {object} params - 包含参数信息的对象。
 * @returns {Promise<any>} - 返回最后一个操作的结果。
 */
async function main(config, params) {
  // 1. 从索引页面抓取代码和详情页 URL.
  const indexPage = await doAction({
    kind: "crawl",
    browser: config.browser,
    urls: [config.index],
    fields: {
      [config.index]: ["code", "url"], // 从索引页面抓取 code 和 url
    },
  });

  // 提取抓取结果中的第一个 URL 的数据
  const fundList = indexPage[config.index];

  // 定义 LLM 需要结构化的数据类型
  const prompts = {
    fundName: "基金名称",
    fundType: "基金类型",
    fundSize: "基金规模",
    fundManager: "基金经理",
    fundCompany: "基金公司",
    fundEstablishmentDate: "基金成立日期",
    fundNetValue: "基金净值",
    fundNetValueDate: "基金净值日期",
    fundAccumulativeNetValue: "基金累计净值",
    fundAccumulativeNetValueDate: "基金累计净值日期",
    fundOneYearReturnRate: "基金近一年收益率",
    fundOneYearReturnRateRanking: "基金近一年收益率排名",
    fundOneYearSimilarAverageReturnRate: "基金近一年同类平均收益率",
    fundOneYearSimilarAverageReturnRateRanking: "基金近一年同类平均收益率排名",
  };

  // 2. 抓取基金详情.
  const fundDetails = {};
  for (const fund of fundList) {
    const url = fund.url;
    fundDetails[url] = {
      kind: "crawl",
      browser: config.browser,
      urls: [url],
      llm: config.llm01,
      fields: {
        [url]: Object.keys(prompts), // 使用 prompts 的 key 作为字段名
      },
    };
  }

  const crawlResults = {};
  for (const url in fundDetails) {
    crawlResults[url] = await doAction(fundDetails[url]);
  }

  // 3. 提取并格式化抓取的数据.
  const csvHeader = Object.values(prompts).join(",") + "\n"; // 使用 prompts 的 value 作为 CSV header

  let csvContent = csvHeader;
  for (const url in crawlResults) {
    const detail = crawlResults[url][url][0]; // 获取抓取结果中的第一个元素
    const row = Object.keys(prompts)
      .map((key) => detail[key] || "")
      .join(",");
    csvContent += row + "\n";
  }

  // 4. 保存数据到 CSV 文件.
  const fileResult = await doAction({
    kind: "file",
    method: "WRITE",
    path: "fund_info.csv",
    body: csvContent,
  });

  return fileResult;
}
