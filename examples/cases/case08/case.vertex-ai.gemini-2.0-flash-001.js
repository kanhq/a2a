// written by LLM provider: vertex-ai model: gemini-2.0-flash-001
async function main(config, params) {
  // 1. Analyze key links in the supply chain
  // Search the web for information about Tesla's supply chain, focusing on key components like batteries, semiconductors, and raw materials.
  const supplyChainSearch = await doAction({
    kind: "web_search",
    browser: config.browser,
    query: "Tesla supply chain analysis batteries semiconductors raw materials",
    provider: "bing",
    pages: 3,
  });

  // 2. Assess market demand and competitive environment
  // Search the web for information on Tesla's market demand, market share, and competitors.
  const marketAnalysisSearch = await doAction({
    kind: "web_search",
    browser: config.browser,
    query: "Tesla market demand market share competitors",
    provider: "bing",
    pages: 3,
  });

  // 3. Risk management strategies
  // Search the web for information about Tesla's risk management strategies.
  const riskManagementSearch = await doAction({
    kind: "web_search",
    browser: config.browser,
    query: "Tesla supply chain risk management strategies",
    provider: "bing",
    pages: 3,
  });

  // 4. Recommended improvement measures
  // Search the web for articles suggesting improvements to Tesla's supply chain.
  const improvementMeasuresSearch = await doAction({
    kind: "web_search",
    browser: config.browser,
    query: "Tesla supply chain improvement recommendations",
    provider: "bing",
    pages: 3,
  });

  // Use LLM to generate the report based on the search results.
  const report = await doAction({
    kind: "llm",
    connection: config.llm,
    sysPrompt: `You are a research analyst. Based on the information provided, write a concise and informative report on "Tesla's Supply Chain and Market Analysis". Include the following sections:
    1. Analysis of key links in the supply chain (mentioning batteries, semiconductors, raw materials, etc.)
    2. Assessment of market demand and competitive environment
    3. Risk management strategies
    4. Recommended improvement measures

    Format the report in markdown for readability.`,
    userPrompt: `Supply Chain Information: ${JSON.stringify(supplyChainSearch)}\n\nMarket Analysis Information: ${JSON.stringify(marketAnalysisSearch)}\n\nRisk Management Information: ${JSON.stringify(riskManagementSearch)}\n\nImprovement Measures Information: ${JSON.stringify(improvementMeasuresSearch)}`,
  });

  // Save the report to a file.
  const fileResult = await doAction({
    kind: "file",
    method: "WRITE",
    path: "teslasupplychain_analysis.md",
    body: report,
  });

  return fileResult;
}
