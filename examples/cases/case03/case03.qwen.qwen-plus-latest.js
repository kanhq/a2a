export async function main(config, params) {
  // 从配置中获取必要的信息
  const ocrUrl = config.ocr_url;
  const ocrApiKey = config.ocr_api_key;
  const emailAccount = config.email_account;
  const emailSavePointPath = config.email_save_point;

  // 读取 'email_save_point' 配置的 JSON 文件
  const lastIdFileAction = {
    kind: 'file',
    method: 'READ',
    path: emailSavePointPath,
    overrideResultMimeType: 'application/json'
  };
  const lastIdFileResult = await doAction(lastIdFileAction);
  const lastId = lastIdFileResult.lastId || 0;

  // 获取新邮件
  const emailAction = {
    kind: 'email',
    method: 'RECV',
    account: emailAccount,
    folder: 'INBOX',
    last_id: lastId
  };
  const emails = await doAction(emailAction);

  // 处理每封新邮件
  for (const email of emails) {
    if (email.from === 'tom@microsoft.com' && email.attachments.length > 0) {
      for (const attachment of email.attachments) {
        // 读取附件内容
        const attachmentFileAction = {
          kind: 'file',
          method: 'READ',
          path: attachment
        };
        const attachmentContent = await doAction(attachmentFileAction);

        // 检查附件是否为图片或 PDF
        const isImageOrPdf = attachmentContent.startsWith('data:image/') || attachmentContent.startsWith('data:application/pdf');
        if (isImageOrPdf) {
          // 提交到 OCR 服务
          const ocrAction = {
            kind: 'http',
            method: 'POST',
            url: ocrUrl,
            headers: {
              'Content-Type': 'application/json',
              'Authorization': `Bearer ${ocrApiKey}`
            },
            body: {
              image: attachmentContent
            }
          };
          await doAction(ocrAction);
        }
      }
    }
  }

  // 更新 'lastId' 到 'email_save_point' 配置的 JSON 文件
  const newLastIdFileAction = {
    kind: 'file',
    method: 'WRITE',
    path: emailSavePointPath,
    body: { lastId: emails[emails.length - 1]?.id || lastId },
    overrideResultMimeType: 'application/json'
  };
  await doAction(newLastIdFileAction);

  // 返回最后处理的邮件 ID
  return { lastProcessedEmailId: emails[emails.length - 1]?.id || lastId };
}
