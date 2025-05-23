## 抓取代码列表

从 `config.index` 配置的索引页面通过 'config.llm01' 配置的 LLM 抓取详情页面的代码和详情页 URL, 其中数据节点的选择器是 '#oTable', 它将得到一个

```typescript
{
  "code": "000001",
  "url": "https://fund.eastmoney.com/000001.html"
}
```

的数组, 代码是基金代码, URL 是基金详情页面的 UR, 每个 URL 都是 `https://fund.eastmoney.com/${code}.html` 的形式, 其中 `code` 是基金代码。

## 抓取基金详情

- 对上述步骤得到的数组, 对每个基金详情页面进行抓取, 抓取的数据包括:
  - 基金名称
  - 基金类型
  - 基金规模
  - 基金经理
  - 基金公司
  - 基金成立日期
  - 基金净值
  - 基金净值日期
  - 基金累计净值
  - 基金累计净值日期
  - 基金近一年收益率
  - 基金近一年收益率排名
  - 基金近一年同类平均收益率
  - 基金近一年同类平均收益率排名

等信息, 并将抓取的数据保存到 `fund_info.csv` 文件中。

- 根据以上的需要，定义 `prompts` 对象，并将它传递给 `crawl`, 它定义了 LLM 需要结构化的数据类型。
- 抓取所需要的浏览器配置可以从 `config.browser` 中获取。
