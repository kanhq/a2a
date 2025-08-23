// written by LLM provider: qwen model: qwen-plus-latest
async function main(config, params) {
  // 读取保存的最后邮件ID
  const savePointFile = await doAction({
    kind: "file",
    method: "READ",
    path: config.email_save_point
  });
  const lastId = savePointFile.lastId || 0;

  // 获取新邮件
  const emails = await doAction({
    kind: "email",
    method: "RECV",
    account: config.email_account,
    last_id: lastId
  });

  // 用于存储通知详情的数组
  const details = [];
  
  // 处理每封邮件
  for (const email of emails) {
    // 检查是否为指定发件人且包含附件
    if (email.from === 'tom@vendor.com' && email.attachments && email.attachments.length > 0) {
      // 处理每个附件
      for (const attachmentPath of email.attachments) {
        // 读取附件内容
        const attachmentData = await doAction({
          kind: "file",
          method: "READ",
          path: attachmentPath
        });

        // 根据文件类型进行base64编码
        let mimeType = "image/*";
        if (attachmentPath.endsWith(".pdf")) {
          mimeType = "application/pdf";
        }
        
        const base64Data = await doAction({
          kind: "enc",
          methods: ["base64"],
          data: attachmentData
        });

        const dataUrl = `data:${mimeType};base64,${base64Data}`;

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

        // 提取结果信息
        const resultInfo = {
          documentType: ocrResult.body.documentType,
          sn: ocrResult.body.invoiceNumber || ocrResult.body.receiptNumber,
          amount: ocrResult.body.amount,
          vendor: "vendor.com"
        };

        // 根据文档类型调用相应的OA服务
        if (resultInfo.documentType === "invoice") {
          await doAction({
            kind: "http",
            method: "POST",
            url: config.oa_invoice,
            body: ocrResult.body
          });
        } else if (resultInfo.documentType === "receipt") {
          await doAction({
            kind: "http",
            method: "POST",
            url: config.oa_receipt,
            body: ocrResult.body
          });
        }

        // 添加到通知详情
        details.push(resultInfo);
      }
    }
  }

  // 更新最后处理的邮件ID
  await doAction({
    kind: "file",
    method: "WRITE",
    path: config.email_save_point,
    body: { lastId: emails[emails.length - 1].id }
  });

  // 如果有处理结果，发送钉钉通知
  if (details.length > 0) {
    const currentDate = new Date().toISOString().split('T')[0];
    
    // 构建通知内容
    let detailItems = "";
    for (const detail of details) {
      detailItems += `- Type: ${detail.documentType} No.: ${detail.sn} Amount: ${detail.amount}\n`;
    }
    
    const notificationContent = `# ${details[0].vendor} New Email Processing Completed\n\n# Date: ${currentDate}\n\n# Details\n\n${detailItems}`;

    // 发送钉钉通知
    await doAction({
      kind: "notify",
      url: config.dingtalk,
      message: notificationContent
    });
  }

  // 返回最后的处理结果
  return details;
}
