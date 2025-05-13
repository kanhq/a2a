export async function main(config, params) {
  // 从 'email_save_point' 配置的 JSON 文件中，读取 'lastId' 的值
  const lastId = JSON.parse(await doAction({
    kind: "file",
    method: "READ",
    path: config.email_save_point,
  })).lastId;

  // 从 'email_account' 配置的账户，获取新邮件
  const emails = await doAction({
    kind: "email",
    method: "RECV",
    account: config.email_account,
    last_id: lastId,
  });

  // 遍历邮件
  for (const email of emails) {
    // 如果邮件发件人是 'tom@microsoft.com' 并且有附件
    if (email.from === "tom@microsoft.com" && email.attachments.length > 0) {
      // 遍历附件
      for (const attachment of email.attachments) {
        // 如果附件是图片或 PDF
        if (attachment.endsWith(".jpg") || attachment.endsWith(".png") || attachment.endsWith(".pdf")) {
          // 将附件内容转为 DataURL
          const imageDataURL = await doAction({
            kind: "file",
            method: "READ",
            path: attachment,
            overrideResultMimeType: "data:image/png;base64",
          });

          // 提交给 OCR 服务
          const ocrResult = await doAction({
            kind: "http",
            method: "POST",
            url: config.ocr_url,
            headers: {
              "Content-Type": "application/json",
              Authorization: config.ocr_api_key,
            },
            body: {
              image: imageDataURL,
            },
          });
        }
      }
    }
  }

  // 保存 'lastId' 的值到 'email_save_point' 配置的 JSON 文件中
  await doAction({
    kind: "file",
    method: "WRITE",
    path: config.email_save_point,
    body: JSON.stringify({
      lastId: emails.length > 0 ? emails[emails.length - 1].id : lastId,
    }),
  });

  return emails.length;
}
