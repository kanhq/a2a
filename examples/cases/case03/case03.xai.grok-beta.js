// written by LLM provider: xai model: grok-beta
export async function main(config, params) {
  // 从配置文件中读取 lastId
  let fileAction = {
    kind: "file",
    method: "READ",
    path: config.email_save_point,
    overrideResultMimeType: "application/json"
  };
  let fileResult = await doAction(fileAction);
  let lastId = fileResult.lastId || 0;

  // 获取新邮件
  let emailAction = {
    kind: "email",
    method: "RECV",
    account: config.email_account,
    folder: "INBOX",
    last_id: lastId
  };
  let emails = await doAction(emailAction);

  let ocrPromises = [];
  let newLastId = lastId;

  for (let email of emails) {
    if (email.from === "tom@vendor.com" && email.attachments.length > 0) {
      for (let attachment of email.attachments) {
        // 读取附件内容并准备 OCR 请求
        let fileReadAction = {
          kind: "file",
          method: "READ",
          path: attachment,
          overrideResultMimeType: "image/png"
        };
        let attachmentContent = await doAction(fileReadAction);
        let ocrAction = {
          kind: "http",
          method: "POST",
          url: config.ocr_url,
          headers: {
            "Content-Type": "application/json",
            "Authorization": config.ocr_api_key
          },
          body: {
            image: attachmentContent
          }
        };
        ocrPromises.push(doAction(ocrAction));
      }
    }
    if (email.id > newLastId) newLastId = email.id;
  }

  // 等待所有 OCR 请求完成
  let ocrResults = await Promise.all(ocrPromises);

  // 处理 OCR 结果
  let oaActions = [];
  let notificationDetails = [];

  for (let ocrResult of ocrResults) {
    if (ocrResult.documentType === "invoice") {
      oaActions.push({
        kind: "http",
        method: "POST",
        url: config.oa_invoice,
        body: ocrResult
      });
      notificationDetails.push({
        documentType: "invoice",
        sn: ocrResult.invoiceNumber,
        amount: ocrResult.amount
      });
    } else if (ocrResult.documentType === "receipt") {
      oaActions.push({
        kind: "http",
        method: "POST",
        url: config.oa_receipt,
        body: ocrResult
      });
      notificationDetails.push({
        documentType: "receipt",
        sn: ocrResult.receiptNumber,
        amount: ocrResult.amount
      });
    }
  }

  // 执行 OA 服务调用
  await Promise.all(oaActions);

  // 保存新的 lastId 到配置文件
  let saveAction = {
    kind: "file",
    method: "WRITE",
    path: config.email_save_point,
    body: JSON.stringify({ lastId: newLastId }),
    overrideResultMimeType: "application/json"
  };
  await doAction(saveAction);

  // 发送通知
  let vendor = "vendor.com";
  let date = new Date().toISOString().split('T')[0];
  let markdownContent = `### ${vendor} 新邮件处理完成\n\n### 日期: ${date}\n\n### 明细\n\n`;
  notificationDetails.forEach(detail => {
    markdownContent += `- 类型: ${detail.documentType} 票号: ${detail.sn} 金额: ${detail.amount}\n`;
  });

  let notifyAction = {
    kind: "notify",
    url: config.dingtalk,
    message: markdownContent,
    title: `${vendor} 新邮件处理完成`
  };
  let notifyResult = await doAction(notifyAction);

  return notifyResult;
}
