// written by LLM provider: model-scope model: Qwen/Qwen3-Coder-30B-A3B-Instruct
async function main(config, params) {
  // 读取保存的最后邮件ID
  const savePointFile = config.email_save_point;
  let lastId = 0;
  
  try {
    const savedData = await doAction({
      kind: "file",
      method: "READ",
      path: savePointFile
    });
    
    if (savedData && savedData.lastId !== undefined) {
      lastId = savedData.lastId;
    }
  } catch (e) {
    // 如果文件不存在或读取失败，使用默认值0
  }
  
  // 获取新邮件
  const emailAccount = config.email_account;
  const emails = await doAction({
    kind: "email",
    method: "RECV",
    account: emailAccount,
    last_id: lastId
  });
  
  // 存储处理结果
  const results = [];
  
  // 处理每封邮件
  for (const email of emails) {
    // 更新最后ID
    if (email.id > lastId) {
      lastId = email.id;
    }
    
    // 检查发件人是否为tom@vendor.com
    if (email.from.includes("tom@vendor.com") && email.attachments && email.attachments.length > 0) {
      // 处理附件
      for (const attachmentPath of email.attachments) {
        // 检查附件类型（图像或PDF）
        const fileName = attachmentPath.split('/').pop();
        const ext = fileName.split('.').pop().toLowerCase();
        
        if (ext === 'jpg' || ext === 'jpeg' || ext === 'png' || ext === 'pdf') {
          // 读取附件内容
          const attachmentContent = await doAction({
            kind: "file",
            method: "READ",
            path: attachmentPath
          });
          
          // 将附件转换为DataURL格式
          let dataUrl = "";
          if (typeof attachmentContent === "string") {
            dataUrl = attachmentContent;
          } else {
            // 如果是二进制数据，需要进行base64编码
            const encoded = await doAction({
              kind: "enc",
              methods: ["base64"],
              data: JSON.stringify(attachmentContent)
            });
            dataUrl = `data:${ext === 'pdf' ? 'application/pdf' : 'image/${ext}'};base64,${encoded}`;
          }
          
          // 调用OCR服务
          const ocrResult = await doAction({
            kind: "http",
            method: "POST",
            url: config.ocr_url,
            headers: {
              "Content-Type": "application/json",
              "Authorization": config.ocr_api_key
            },
            body: {
              image: dataUrl
            }
          });
          
          // 保存结果
          if (ocrResult && ocrResult.documentType) {
            results.push({
              vendor: email.from.split('@')[1],
              documentType: ocrResult.documentType,
              sn: ocrResult.invoiceNumber || ocrResult.receiptNumber,
              amount: ocrResult.amount,
              date: new Date().toISOString().split('T')[0]
            });
            
            // 根据文档类型调用OA服务
            if (ocrResult.documentType === 'invoice') {
              await doAction({
                kind: "http",
                method: "POST",
                url: config.oa_invoice,
                headers: {
                  "Content-Type": "application/json"
                },
                body: ocrResult
              });
            } else if (ocrResult.documentType === 'receipt') {
              await doAction({
                kind: "http",
                method: "POST",
                url: config.oa_receipt,
                headers: {
                  "Content-Type": "application/json"
                },
                body: ocrResult
              });
            }
          }
        }
      }
    }
  }
  
  // 保存最新的邮件ID
  await doAction({
    kind: "file",
    method: "WRITE",
    path: savePointFile,
    body: {
      lastId: lastId
    }
  });
  
  // 发送通知
  if (results.length > 0) {
    let notifyContent = "# New Email Processing Completed\n\n";
    notifyContent += `# Date: ${new Date().toISOString().split('T')[0]}\n\n`;
    notifyContent += "# Details\n\n";
    
    for (const result of results) {
      notifyContent += `- Type: ${result.documentType} No.: ${result.sn} Amount: ${result.amount}\n`;
    }
    
    await doAction({
      kind: "notify",
      url: config.dingtalk,
      message: notifyContent,
      title: "Email Processing Completed"
    });
  }
  
  return results;
}
