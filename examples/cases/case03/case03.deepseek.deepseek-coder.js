export async function main(config, params) {
  // 从 'email_save_point' 配置的 JSON 文件中读取 'lastId' 的值
  const fileAction = {
    kind: "file",
    method: "READ",
    path: config.email_save_point,
  };
  const savePoint = await doAction(fileAction);
  const lastId = savePoint.lastId;

  // 从 'email_account' 配置的账户，获取新邮件
  const emailAction = {
    kind: "email",
    method: "RECV",
    account: config.email_account,
    last_id: lastId,
  };
  const emails = await doAction(emailAction);

  // 对获取到的文件，如果其发件人是 'tom@microsoft.com', 并且有附件，则将附件中的图片或 PDF 提交给 `OCR` 服务
  for (const email of emails) {
    if (email.from === 'tom@microsoft.com' && email.attachments.length > 0) {
      for (const attachment of email.attachments) {
        const fileAction = {
          kind: "file",
          method: "READ",
          path: attachment,
        };
        const fileContent = await doAction(fileAction);

        // 检查文件是否为图片或 PDF
        const isImageOrPdf = fileContent.mimeType.startsWith('image/') || fileContent.mimeType === 'application/pdf';
        if (isImageOrPdf) {
          const ocrAction = {
            kind: "http",
            method: "POST",
            url: config.ocr_url,
            headers: {
              'Content-Type': 'application/json',
              'Authorization': config.ocr_api_key,
            },
            body: {
              image: fileContent.dataUrl,
            },
          };
          await doAction(ocrAction);
        }
      }
    }
  }

  // 保存 'lastId' 的值到 'email_save_point' 配置的 JSON 文件中
  const newLastId = emails.length > 0 ? emails[emails.length - 1].id : lastId;
  const saveAction = {
    kind: "file",
    method: "WRITE",
    path: config.email_save_point,
    body: { lastId: newLastId },
  };
  await doAction(saveAction);

  return newLastId;
}
