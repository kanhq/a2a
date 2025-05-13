// written by LLM provider: vertex-ai model: gemini-2.0-flash-exp
// 主函数，程序的入口点
async function main(config, params) {
  // 创建一个 FileAction，用于列出文件
  const fileAction = {
    kind: "file",
    method: "LIST",
    path: "case/**/prompt.md", // 指定要列出的文件路径，使用 ** 匹配所有子目录
  };

  // 调用 doAction 函数执行 FileAction，获取文件列表
  const fileResult = await doAction(fileAction);

  // 返回文件列表
  return fileResult;
}
