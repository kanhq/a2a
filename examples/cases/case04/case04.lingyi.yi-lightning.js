export async function main(config, params) {
  // 读取图片文件
  const fileAction = {
    kind: "file",
    method: "READ",
    path: params.file, // 使用传入参数中的文件路径
  };
  const fileResult = await doAction(fileAction);

  // 调用 LLM 进行图片识别，并要求返回指定格式的 JSON 结果
  const llmAction = {
    kind: "llm",
    connection: config.llm01, // 使用配置中的 llm01 连接信息
    userImage: fileResult, // 将读取的图片文件传递给 LLM
    sysPrompt: `请你对提供的图片进行识别，并返回一个包含以下字段的JSON对象：
    - name: 识别出的名字（如果有）
    - address: 识别出的地址（如果有）
    - phone: 识别出的电话号码（如果有）
    - email: 识别出的电子邮件地址（如果有）。
    请确保返回的JSON格式严格遵循以下TypeScript类型定义：
    type Result = {
      name?: string;
      address?: string;
      phone?: string;
      email?: string;
    };`,
    // 我们也可以指定让 LLM 以 JSON 格式返回结果
    overrideResultMimeType: "application/json",
  };

  // 调用 LLM 模型并返回结果
  return await doAction(llmAction);
}
