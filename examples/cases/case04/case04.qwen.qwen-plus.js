// written by LLM provider: qwen model: qwen-plus
export async function main(config, params) {
  // 构建 LLM 动作对象
  const action = {
    kind: "llm",
    connection: config.llm01, // 使用配置中的 llm01 连接
    overrideResultMimeType: "application/json", // 指定结果为 JSON 格式
    sysPrompt: "你将收到一张图片，请识别图片中的信息并以指定的 JSON 结构返回。",
    userPrompt: "请识别这张图片中的信息，并按照以下结构返回：\n" +
                 "{\n" +
                 "  \"name\": \"\",\n" +
                 "  \"address\": \"\",\n" +
                 "  \"phone\": \"\",\n" +
                 "  \"email\": \"\"\n" +
                 "}",
    userImage: params.file // 使用参数中的文件作为图片
  };

  // 执行 LLM 动作并返回结果
  return await doAction(action);
}
