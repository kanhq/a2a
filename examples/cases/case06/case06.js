// written by LLM provider: vertex-ai model: gemini-2.0-flash-exp
async function main(config, params) {
  // 定义 prompts 对象，用于 LLM 结构化数据
  const prompts = {
    fund_info: {
      "基金名称": "fundName",
      "基金类型": "fundType",
      "基金规模": "fundSize",
      "基金经理": "fundManager",
      "基金公司": "fundCompany",
      "成立日期": "establishDate",
      "单位净值": "netValue",
      "净值日期": "netValueDate",
      "累计净值": "totalNetValue",
      "累计净值日期": "totalNetValueDate",
      "近1年收益": "oneYearReturn",
      "近1年排名": "oneYearRank",
      "同类平均": "oneYearAvgReturn",
      "同类排名": "oneYearAvgRank"
    }
  };


  // 从配置中获取索引页面的 URL
  const indexUrl = config.index;
  // 使用 crawl 抓取索引页面，提取基金代码和详情页 URL
  const indexResult = await doAction({
    kind: "crawl",
    browser: config.browser,
    urls: [
      {
        url: indexUrl,
        selector: '#oTable'
      }
    ],
    llm: config.llm01,
    fields: ["code", "url"]
  });

  // 从抓取结果中提取基金代码和 URL
  const fundList = indexResult[indexUrl].map(item => {
    url: `https://fund.eastmoney.com/${item.url}`
  });

  console.log(fundList);

  //   // 定义字段映射
  //   const fields = {
  //       [`https://fund.eastmoney.com/*.html`]: Object.values(prompts.fund_info)
  //   };

  // // 使用 crawl 抓取每个基金详情页面的数据
  // const detailResult = await doAction({
  //   kind: "crawl",
  //   browser: config.browser,
  //     urls: fundList.map(item => item.url),
  //       fields: fields,
  //       llm: config.llm01,
  //   });


  // // 将抓取的数据转换为 CSV 格式
  // const csvHeader = Object.keys(prompts.fund_info).join(",") + "\n";
  // const csvRows = Object.values(detailResult).map(item => {
  //      if(typeof item === 'string'){
  //           return '';
  //      }
  //     const values = Object.values(prompts.fund_info).map(key => item[key] || '').join(',');
  //     return values
  //   });

  // const csvData = csvHeader + csvRows.join('\n');


  // // 将 CSV 数据写入文件
  // return await doAction({
  //   kind: "file",
  //   method: "WRITE",
  //   path: "fund_info.csv",
  //   body: csvData,
  //   overrideResultMimeType: 'text/csv'
  // });
}
