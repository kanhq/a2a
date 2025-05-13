// written by LLM provider: xai model: grok-beta
export async function main(config, params) {
  const llmAction = {
    kind: "llm",
    connection: config.llm01,
    userImage: params.file,
    userPrompt: "请识别图片中的内容，并返回包含以下结构的JSON格式结果：\n```typescript\n{\n  name?: string;\n  address?: string;\n  phone?: string;\n  email?: string;\n}\n```\n请确保返回的结果符合上述结构。",
    overrideResultMimeType: "application/json"
  };

  const result = await doAction(llmAction);
  return result;
}
