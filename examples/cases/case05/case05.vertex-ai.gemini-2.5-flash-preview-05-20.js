// written by LLM provider: vertex-ai model: gemini-2.5-flash-preview-05-20
/**
 * 从所有 invoice*.pdf 文件中抽取第二页，并将其合并到 all_invoices.pdf 中。
 * @param {object} config - 应用程序配置对象。
 * @param {object} params - 应用程序参数对象。
 * @returns {Promise<string>} - 合并操作的结果消息。
 */
async function main(config, params) {
  // 1. 列出所有 invoice*.pdf 文件
  // 使用 ShellAction 执行 ls 命令来查找匹配的文件
  const listFilesResult = await doAction({
    kind: "shell",
    command: "ls",
    args: ["invoice*.pdf"]
  });

  // 将 ls 命令的输出按换行符分割成文件名的数组，并过滤掉空字符串
  const pdfFiles = listFilesResult.split('\n').filter(name => name.trim() !== '');

  const extractedPages = []; // 用于存储抽取出的临时文件路径
  let fileIndex = 0; // 用于生成唯一的临时文件名

  // 2. 遍历每个找到的 PDF 文件，抽取其第二页
  for (const file of pdfFiles) {
    const trimmedFile = file.trim(); // 清除文件名两端的空白符
    if (trimmedFile) { // 确保文件名不为空
      // 构建临时输出文件名，例如 temp_extracted_invoice_0_page2.pdf
      const outputFileName = `temp_extracted_invoice_${fileIndex}_page2.pdf`;
      extractedPages.push(outputFileName); // 将临时文件名添加到列表中

      // 使用 mutool create 命令抽取 PDF 的第二页
      // mutool create -o <output_file> <input_file> <page_number>
      await doAction({
        kind: "shell",
        command: "mutool",
        args: ["create", "-o", outputFileName, trimmedFile, "2"] // "2" 表示抽取第二页
      });
      fileIndex++; // 递增索引以确保下一个临时文件名的唯一性
    }
  }

  // 3. 合并所有抽取的第二页
  if (extractedPages.length > 0) {
    // 构造 mutool merge 命令的参数数组
    // mutool merge -o <output_file> <input_file1> <input_file2> ...
    const mergeArgs = ["merge", "-o", "all_invoices.pdf", ...extractedPages];

    // 执行合并操作
    const mergeResult = await doAction({
      kind: "shell",
      command: "mutool",
      args: mergeArgs
    });

    // 返回合并操作的结果
    return mergeResult;
  } else {
    // 如果没有找到任何匹配的 PDF 文件或未能抽取任何页面，则返回相应的消息
    return "没有找到任何 invoice*.pdf 文件，或未能抽取任何页面。";
  }
  // 注意：根据要求，这里不进行临时文件的清理，也没有错误处理（try/catch）。
}
