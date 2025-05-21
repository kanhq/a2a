## Scraping Code List

From the index page configured in `config.index`, use the LLM configured in 'config.llm01' to scrape the codes and detail page URLs. The data node selector is '#oTable', which will yield an array of the following structure:

```typescript
{
  "code": "000001",
  "url": "https://fund.eastmoney.com/000001.html"
}
```

Here, 'code' refers to the fund code, and 'url' is the URL of the fund's detail page. Each URL follows the format `https://fund.eastmoney.com/${code}.html`, where `code` is the fund code.

## Scraping Fund Details

- For the array obtained in the previous step, scrape each fund's detail page. The data to be scraped includes:
  - Fund Name
  - Fund Type
  - Fund Size
  - Fund Manager
  - Fund Company
  - Fund Inception Date
  - Fund Net Asset Value (NAV)
  - Fund NAV Date
  - Fund Accumulated Net Asset Value
  - Fund Accumulated NAV Date
  - Fund One-Year Return
  - Fund One-Year Return Ranking
  - Fund One-Year Peer Average Return
  - Fund One-Year Peer Average Return Ranking

and other information. Save the scraped data to `fund_info.csv`.

- Based on the above requirements, define a `prompts` object and pass it to `crawl`. This object specifies the data types that the LLM needs to structure.
- The necessary browser configuration for scraping can be retrieved from `config.browser`.