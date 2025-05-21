// written by LLM provider: qwen model: qwen-plus-latest
async function main(config, params) {
  // 查找所有名为 invoice*.pdf 的文件
  const findInvoiceFiles = {
    kind: "shell",
    command: "find",
    args: [".", "-name", "invoice*.pdf"]
  };
  const invoiceFiles = await doAction(findInvoiceFiles);
  const invoiceFileList = invoiceFiles.split("\n").filter(file => file.trim() !== "");

  // 创建一个临时目录用于存放抽取的第二页
  const tempDir = "/tmp/invoice_pages";
  await doAction({
    kind: "shell",
    command: "mkdir",
    args: ["-p", tempDir]
  });

  // 对每个发票文件进行处理，抽取第二页并保存为独立的PDF文件
  for (const file of invoiceFileList) {
    const filename = file.replace("./", "").replace(".pdf", "");
    const outputFile = `${tempDir}/${filename}_page2.pdf`;

    // 使用 mutool extract 抽取第二页
    await doAction({
      kind: "shell",
      command: "mutool",
      args: ["extract", "-p", "2", file, outputFile]
    });
  }

  // 将所有抽取的第二页合并成一个PDF文件
  const mergeCommand = `gs -dBATCH -dNOPAUSE -q -sDEVICE=pdfwrite -sOutputFile=${tempDir}/all_invoices.pdf ${tempDir}/*.pdf`;
  await doAction({
    kind: "shell",
    command: "bash",
    args: ["-c", mergeCommand]
  });

  // 将生成的合并后的PDF移动到当前目录
  await doAction({
    kind: "shell",
    command: "mv",
    args: [`${tempDir}/all_invoices.pdf`, "./all_invoices.pdf"]
  });

  // 返回最终的合并文件路径
  return "./all_invoices.pdf";
}
