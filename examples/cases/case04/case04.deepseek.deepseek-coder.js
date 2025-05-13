// written by LLM provider: deepseek model: deepseek-coder
export async function main(config, params) {
  // 定义 LLM 动作
  const llmAction = {
    kind: "llm",
    connection: config.llm01, // 使用配置中的 llm01 连接
    sysPrompt: "请识别图片中的信息，并以 JSON 格式返回结果。返回的 JSON 必须符合以下结构：\n```typescript\ntype Result = {\n  name?: string;\n  address?: string;\n  phone?: string;\n  email?: string;\n};\n```",
    userImage: params.file, // 使用参数中的图片
    overrideResultMimeType: "application/json" // 指定返回结果的 MIME 类型为 JSON
  };

  // 执行 LLM 动作
  const llmResult = await doAction(llmAction);

  // 返回 LLM 的结果
  return llmResult;
}
