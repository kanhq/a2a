// written by LLM provider: azure-openai model: gpt-4o
export async function main(config, params) {
  // 读取 'email_save_point' 文件获取上次处理的最后邮件ID
  const emailSavePoint = await doAction({
    kind: "file",
    method: "READ",
    path: config.email_save_point,
    overrideResultMimeType: "application/json"
  });

  const lastId = emailSavePoint.lastId;

  // 获取新邮件
  const emails = await doAction({
    kind: "email",
    method: "RECV",
    account: config.email_account,
    last_id: lastId
  });

  const newEmails = emails.filter(email => email.from === 'tom@vendor.com' && email.attachments && email.attachments.length > 0);

  // 初始化通知明细
  let notificationDetails = [];

  for (const email of newEmails) {
    for (const attachment of email.attachments) {
      // 检查文件类型
      if (attachment.endsWith('.png') || attachment.endsWith('.jpg') || attachment.endsWith('.jpeg') || attachment.endsWith('.pdf')) {
        
        // 读取附件内容为 DataURL
        const fileContent = await doAction({
          kind: "file",
          method: "READ",
          path: attachment
        });

        // 调用 OCR 服务
        const ocrResult = await doAction({
          kind: "http",
          method: "POST",
          url: config.ocr_url,
          headers: {
            "Content-Type": "application/json",
            "Authorization": config.ocr_api_key
          },
          body: {
            image: fileContent
          }
        });
        
        const documentType = ocrResult.documentType;
        const sn = documentType === 'invoice' ? ocrResult.invoiceNumber : ocrResult.receiptNumber;
        const amount = ocrResult.amount;

        // 根据文档类型调用相应的 OA 服务
        if (documentType === 'invoice') {
          await doAction({
            kind: "http",
            method: "POST",
            url: config.oa_invoice,
            headers: {
              "Content-Type": "application/json"
            },
            body: ocrResult
          });
        } else if (documentType === 'receipt') {
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

        // 添加通知明细
        notificationDetails.push(`- 类型: ${documentType} 票号: ${sn} 金额: ${amount}`);
      }
    }
  }

  // 更新 'email_save_point' 文件中的 lastId
  const newLastId = Math.max(...emails.map(email => email.id));
  if (newLastId > lastId) {
    await doAction({
      kind: "file",
      method: "WRITE",
      path: config.email_save_point,
      body: JSON.stringify({ lastId: newLastId })
    });
  }

  // 发送钉钉通知
  if (notificationDetails.length > 0) {
    const vendor = new URL(emails[0].from).hostname;
    const todayDate = new Date().toLocaleDateString();

    const notificationContent = `
# ${vendor} 新邮件处理完成

# 日期: ${todayDate}

# 明细

${notificationDetails.join('\n')}
`;

    await doAction({
      kind: "http",
      method: "POST",
      url: config.dingtalk,
      headers: {
        "Content-Type": "application/json"
      },
      body: {
        msgtype: "markdown",
        markdown: {
          title: `${vendor} 新邮件处理完成`,
          text: notificationContent
        }
      }
    });
  }

  return notificationDetails;
}
