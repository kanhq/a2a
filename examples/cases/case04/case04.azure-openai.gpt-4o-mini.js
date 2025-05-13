export async function main(config, params) {
    // 定义 LLM action
    const llmAction = {
        kind: "llm",
        connection: config.llm01, // 使用配置中的 LLM 连接
        sysPrompt: "请识别以下图片并返回包含姓名、地址、电话和电子邮件的JSON格式结果",
        userImage: params.file, // 图片参数
        overrideResultMimeType: "application/json" // 指定返回 JSON 格式
    };

    // 调用 LLM 接口并返回结果
    const result = await doAction(llmAction);
    return result; // 返回识别结果
}
