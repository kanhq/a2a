// written by LLM provider: vertex-ai model: gemini-1.5-pro-002
/**
 * 调用 `llm01` 配置的大模型， 让它对参数中 `file` 代表的图片进行识别，并让它用 `JSON` 格式返回结果
 * 返回的 JSON 中，必须符合以下的结构
 *
 * ```typescript
 * type Result = {
 *   name?: string;
 *   address?: string;
 *   phone?: string;
 *   email?: string;
 * };
 * ```
 */
export async function main(config, params) {
  // 获取 llm01 的配置
  const llmConnection = config.llm01;

  // 定义系统提示词，要求返回 JSON 格式的结果，并指定 JSON 的结构
  const sysPrompt = `你是一个 OCR 工具，你需要识别用户提供的图片，并提取图片中的信息，用 JSON 格式返回结果。
  JSON 的结构如下：
  \`\`\`typescript
  type Result = {
    name?: string;
    address?: string;
    phone?: string;
    email?: string;
  };
  \`\`\`
  `;

  // 定义用户提示词
  const userPrompt = "请识别这张图片";

  // 创建 LLM Action
  const llmAction = {
    kind: "llm",
    connection: llmConnection,
    sysPrompt: sysPrompt,
    userPrompt: userPrompt,
    userImage: params.file, // 将参数中的 file 传递给 userImage
    overrideResultMimeType: "application/json", // 指定返回结果为 JSON 格式
  };

  // 调用 doAction 执行 LLM Action
  const result = await doAction(llmAction);

  // 返回 LLM Action 的结果
  return result;
}
