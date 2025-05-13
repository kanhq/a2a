// written by LLM provider: vertex-ai model: gemini-1.5-flash-002
//  调用 llm01 配置的大模型，让它对参数中 file 代表的图片进行识别，并让它用 JSON 格式返回结果
// 返回的 JSON 中，必须符合以下的结构
// type Result = {
//   name?: string;
//   address?: string;
//   phone?: string;
//   email?: string;
// };

export async function main(config, params) {
  // 获取 llm01 配置
  const llmConfig = config.llm01;

  // 构造 LLM action
  const llmAction = {
    kind: "llm",
    connection: llmConfig.connection,
    sysPrompt: `请识别图片中的信息，并以 JSON 格式返回结果。\
    结果必须包含以下字段：name, address, phone, email。\
    如果图片中不存在某个字段的信息，则将该字段的值设为 undefined。\
    请确保返回的 JSON 结构符合以下规范：\
    {\n  "name": string,\n  "address": string,\n  "phone": string,\n  "email": string\n}`,
    userPrompt: "",
    userImage: params.file,
    overrideResultMimeType: "application/json",
  };

  // 调用 doAction 执行 LLM action
  const result = await doAction(llmAction);

  // 返回结果
  return result;
}
