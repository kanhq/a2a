// written by LLM provider: deepseek model: deepseek-chat
export async function main(config, params) {
  // 读取 'lastId' 的值
  const fileAction = {
    kind: "file",
    method: "READ",
    path: config.email_save_point,
    overrideResultMimeType: "application/json"
  };
  const lastIdResult = await doAction(fileAction);
  const lastId = lastIdResult.lastId;

  // 获取新邮件
  const emailAction = {
    kind: "email",
    method: "RECV",
    account: config.email_account,
    last_id: lastId
  };
  const emails = await doAction(emailAction);

  // 处理新邮件
  const ocrResults = [];
  for (const email of emails) {
    if (email.from.includes('tom@vendor.com') && email.attachments.length > 0) {
      for (const attachment of email.attachments) {
        const fileAction = {
          kind: "file",
          method: "READ",
          path: attachment,
          overrideResultMimeType: "application/json"
        };
        const fileContent = await doAction(fileAction);

        // 提交附件给 OCR 服务
        const ocrAction = {
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
        };
        const ocrResult = await doAction(ocrAction);
        ocrResults.push(ocrResult);
      }
    }
  }

  // 保存新邮件的 ID
  const newLastId = emails.length > 0 ? emails[emails.length - 1].id : lastId;
  const saveLastIdAction = {
    kind: "file",
    method: "WRITE",
    path: config.email_save_point,
    body: { lastId: newLastId },
    overrideResultMimeType: "application/json"
  };
  await doAction(saveLastIdAction);

  // 处理 OCR 结果
  const notifications = [];
  for (const ocrResult of ocrResults) {
    if (ocrResult.documentType === 'invoice') {
      const oaInvoiceAction = {
        kind: "http",
        method: "POST",
        url: config.oa_invoice,
        headers: {
          "Content-Type": "application/json"
        },
        body: ocrResult
      };
      await doAction(oaInvoiceAction);
      notifications.push(`- 类型: ${ocrResult.documentType} 票号: ${ocrResult.invoiceNumber} 金额: ${ocrResult.amount}`);
    } else if (ocrResult.documentType === 'receipt') {
      const oaReceiptAction = {
        kind: "http",
        method: "POST",
        url: config.oa_receipt,
        headers: {
          "Content-Type": "application/json"
        },
        body: ocrResult
      };
      await doAction(oaReceiptAction);
      notifications.push(`- 类型: ${ocrResult.documentType} 票号: ${ocrResult.receiptNumber} 金额: ${ocrResult.amount}`);
    }
  }

  // 发送通知
  const notifyAction = {
    kind: "notify",
    url: config.dingtalk,
    message: `# ${emails[0].from.split('@')[1]} 新邮件处理完成\n\n# 日期: ${new Date().toISOString().split('T')[0]}\n\n# 明细\n${notifications.join('\n')}`,
    title: "新邮件处理完成"
  };
  return await doAction(notifyAction);
}
