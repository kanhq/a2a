// written by LLM provider: vertex-ai model: gemini-2.5-pro-preview-05-06
async function main(config, params) {
  // 从参数中获取输入的PDF文件路径
  const inputPdfPath = params.input_pdf_path;
  // 从参数中获取输出的PDF文件路径，如果未提供，则基于输入文件名创建一个
  let outputPdfPath = params.output_pdf_path;

  if (!inputPdfPath) {
    throw new Error("params.input_pdf_path is required");
  }

  if (!outputPdfPath) {
    // 从输入路径中提取文件名和目录
    const pathParts = inputPdfPath.split('/');
    const fileNameWithExt = pathParts.pop();
    const dirPath = pathParts.join('/');
    const fileNameParts = fileNameWithExt.split('.');
    const ext = fileNameParts.pop();
    const fileName = fileNameParts.join('.');
    // 构建输出文件名
    outputPdfPath = `${dirPath ? dirPath + '/' : ''}${fileName}_merged.${ext}`;
  }

  // 构建 mutool 命令
  // mutool poster -x 2 input.pdf output.pdf
  // -x 2 表示横向排列2页
  const shellAction = {
    kind: "shell",
    command: "mutool",
    args: [
      "poster",
      "-x",
      "2",
      inputPdfPath,
      outputPdfPath
    ]
  };

  // 执行 Shell 命令
  await doAction(shellAction);

  // 返回新生成的PDF文件路径
  return outputPdfPath;
}
