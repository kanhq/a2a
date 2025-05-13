
// written by LLM provider: vertex-ai model: gemini-2.5-flash-preview-04-17
async function main(config, params) {
  // Search the web for information about MCP usage in LLMs
  const searchResults = await doAction({
    browser: config.browser, // Use the browser from the config
    kind: "web_search",
    query: "Model Context Protocol LLM usage",
    pages: 3, // Search 3 pages for more comprehensive results
    provider: "bing", // Use Bing as the search engine
  });

  // Prepare the context for the LLM from the search results
  // Format the search results into a string that the LLM can easily process
  let context = "Web search results on Model Context Protocol (MCP) usage in LLMs:\n\n";
  if (searchResults && searchResults.length > 0) {
    searchResults.forEach((result, index) => {
      context += `--- Result ${index + 1} ---\n`;
      context += `Title: ${result.title}\n`;
      context += `URL: ${result.url}\n`;
      context += `Body:\n${result.body}\n\n`;
    });
  } else {
    context += "No search results found.";
  }

  // Use the LLM to write a research report based on the search results
  const researchReport = await doAction({
    kind: "llm",
    connection: config.llm, // Use the LLM connection from the config
    sysPrompt: "You are a researcher tasked with writing a comprehensive report on the usage of the Model Context Protocol (MCP) in Large Language Models (LLMs). Synthesize the provided information into a well-structured research paper.",
    userPrompt: `Based on the following context, write a research report about the usage of Model Context Protocol (MCP) in Large Language Models:\n\n${context}\n\nEnsure the report covers key aspects found in the context, such as what MCP is, why it is used, its benefits, challenges, and current applications in the LLM domain.`,
  });

  // Save the generated report to a markdown file
  const saveFileResult = await doAction({
    kind: "file",
    method: "WRITE",
    path: "mcp.md",
    body: researchReport,
  });

  // Return the result of the file saving action
  return saveFileResult;
}
