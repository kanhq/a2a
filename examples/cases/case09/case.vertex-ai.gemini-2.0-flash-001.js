// written by LLM provider: vertex-ai model: gemini-2.0-flash-001
async function main(config, params) {
  // 1. Search the web for "top 50 comprehensive universities in the United States"
  const webSearchAction = {
    kind: "web_search",
    browser: config.browser,
    query: "top 50 comprehensive universities in the United States",
    provider: "bing",
    pages: 1,
  };

  const webSearchResults = await doAction(webSearchAction);

  // 2. Extract the list of universities from the search results
  let universities = [];
  for (const result of webSearchResults) {
    // Assuming the titles of the search results contain the university names
    universities.push(result.title);
  }

  // Limit to top 50, just in case
  universities = universities.slice(0, 50);

  // 3. Search the web for admission requirements for art programs at each university
  let admissionRequirements = {};
  for (const university of universities) {
    const search_query = `${university} art program admission requirements`;
    const artProgramSearchAction = {
      kind: "web_search",
      browser: config.browser,
      query: search_query,
      provider: "bing",
      pages: 1, // Limiting to 1 page for each university to avoid excessive requests
    };

    const artProgramSearchResults = await doAction(artProgramSearchAction);

    // Extract the URLs from the search results
    let urls = [];
    for (const result of artProgramSearchResults) {
      urls.push(result.url);
    }

    admissionRequirements[university] = urls; // Store the URLs for each university
  }

  // 4. Use LLM to summarize and extract the admission requirements from the search results for each university.
  let llmSummaries = {};
  for (const university in admissionRequirements) {
    const urls = admissionRequirements[university];
    let allContent = "";

    // Crawl the URLs and extract the content
    const crawlAction = {
      kind: "crawl",
      browser: config.browser,
      urls: urls,
    };
    const crawlResult = await doAction(crawlAction);

    // Concatenate all crawled content
    for (const url in crawlResult) {
      allContent += crawlResult[url];
    }

    // Use LLM to summarize the admission requirements
    const llmAction = {
      kind: "llm",
      connection: config.llm,
      sysPrompt: `You are an expert at extracting admission requirements for university art programs.  Summarize the admission requirements from the following text. Focus on portfolio requirements, GPA requirements, standardized test requirements, and any other specific requirements for art programs.`,
      userPrompt: `Here is the text to summarize: ${allContent}`,
    };

    const llmResult = await doAction(llmAction);
    llmSummaries[university] = llmResult;
  }

  // 5. Format the admission requirements into a string.
  let fileContent = "";
  for (const university in llmSummaries) {
    fileContent += `${university}:\n${llmSummaries[university]}\n\n`;
  }

  // 6. Save the admission requirements to a file.
  const fileAction = {
    kind: "file",
    method: "WRITE",
    path: "admission_requirements.txt",
    body: fileContent,
  };

  const fileResult = await doAction(fileAction);
  return fileResult; // Return the file write result
}
