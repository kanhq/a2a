// written by LLM provider: vertex-ai model: gemini-2.0-flash-001
async function main(config, params) {
  // 定义保存报告的文件路径
  const reportPath = "mcp.md";

  // 搜索MCP(Model Context Protocol)在大模型领域的使用情况
  const webSearchAction = {
    browser: config.browser,
    kind: "web_search",
    query: "MCP (Model Context Protocol) large language models",
    pages: 5, // 增加搜索页数，获取更多素材
    provider: "bing", // 使用bing搜索引擎
  };

  // 执行网络搜索
  const webSearchResults = await doAction(webSearchAction);

  // 使用LLM生成研究报告
  const llmAction = {
    kind: "llm",
    connection: config.llm,
    sysPrompt: `你是一位人工智能专家，负责编写关于MCP(Model Context Protocol)在大模型领域使用情况的研究报告。
请根据提供的网络搜索结果，撰写一份详细、深入的报告。报告应包括以下内容：
1. MCP 的定义和作用
2. MCP 在大模型领域中的应用场景
3. MCP 的优势和局限性
4. MCP 的未来发展趋势
5. 总结
请使用清晰、简洁的语言，确保报告内容准确、客观。请以markdown格式输出。`,
    userPrompt: `请根据以下网络搜索结果，撰写研究报告：${JSON.stringify(webSearchResults)}`,
  };

  // 执行LLM生成报告
  const report = await doAction(llmAction);

  // 保存报告到文件
  const fileAction = {
    kind: "file",
    method: "WRITE",
    path: reportPath,
    body: report,
  };

  // 执行文件保存操作
  return await doAction(fileAction);
}
