// written by LLM provider: vertex-ai model: gemini-2.0-flash-exp
async function main(config, params) {
  // 1. Web Search for Information
  const searchQueries = [
    "tesla supply chain analysis",
    "tesla market demand",
    "tesla competition",
    "tesla risk management supply chain",
  ];

  const searchResults = [];
  for (const query of searchQueries) {
    const searchAction = {
      kind: "web_search",
      browser: {},
      query: query,
      provider: "bing",
      pages: 3,
    };
    const result = await doAction(searchAction);
    searchResults.push({ query: query, results: result });
  }

  // 2. LLM to generate the report
  let reportContent = "";

  // Introduction
  reportContent += "# Tesla Supply Chain and Market Analysis\n\n";
  reportContent += "This report provides an analysis of Tesla's supply chain, market demand, competitive landscape, risk management strategies, and suggests improvements.\n\n";

  // Key Findings from Web Search
  let searchFindings = "";
  for (const searchResult of searchResults) {
    searchFindings += `## Search Results for "${searchResult.query}"\n`;
    if (searchResult.results && searchResult.results.length > 0) {
      for (const result of searchResult.results) {
        searchFindings += `- [${result.title}](${result.url})\n  ${result.body}\n`;
      }
    } else {
      searchFindings += "No results found.\n";
    }
  }

  // LLM Prompt to Generate the Report Sections based on Web Search Findings
  const llmPrompt = `
  Based on the following search results, please generate a report on Tesla's supply chain and market analysis, covering the following sections:

  1.  **Supply Chain Critical Links Analysis:** Identify and analyze Tesla's critical supply chain components (e.g., battery materials, chips).
  2.  **Market Demand and Competitive Environment Assessment:** Assess Tesla's market demand, potential market size, and the competitive landscape (e.g., BYD, Rivian, traditional automakers).
  3.  **Risk Management Strategies:** Outline Tesla's strategies for mitigating supply chain risks (e.g., diversification, vertical integration).
  4.  **Recommended Improvements:** Suggest improvements to Tesla's supply chain and market strategies.

  Search Results:
  ${searchFindings}

  Please ensure each section is well-structured, clear, and concise.  Use markdown format for headings, lists, and links. Include citations to the relevant URLs from the search results. The report should be easy to read and understand. Assume the reader is familiar with Tesla but needs specific insights into the supply chain and market aspects.
  `;

  const llmAction = {
    kind: "llm",
    connection: {},
    sysPrompt: "You are an expert in automotive industry analysis, especially in supply chain and market strategies.",
    userPrompt: llmPrompt,
  };

  const llmResult = await doAction(llmAction);

  reportContent += llmResult;
  reportContent += "\n\n";

  // 3. Save the report to a file
  const fileAction = {
    kind: "file",
    method: "WRITE",
    path: "tesla_supply_chain_analysis.md",
    body: reportContent,
  };

  const fileResult = await doAction(fileAction);

  return fileResult;
}
