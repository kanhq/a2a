// written by LLM provider: chuanshen model: rendu-2.5
async function main(config, params) {
    const { lastId, emailAccount, ocrUrl, ocrApiKey, oaInvoice, oaReceipt, dingtalk } = config;

    // 1. 从配置文件或 OSS 读取 lastId
    const savedLastId = await readLastIdFromFile(lastId); // 假设 readLastIdFromFile 是您定义的读取 JSON 文件的函数

    // 2. 获取新邮件
    const newEmails = await fetchEmails(emailAccount, savedLastId);

    // 3. 处理新邮件
    for (const email of newEmails) {
        if (email.from === 'tom@vendor.com' && email.attachments.length > 0) {
            const fileData = await attachFileToOCR(email.attachments[0], ocrUrl, ocrApiKey);
            const result = await ocr(fileData);

            if (result.documentType === 'invoice') {
                await submitInvoice(result, oaInvoice);
            } else if (result.documentType === 'receipt') {
                await submitReceipt(result, oaReceipt);
            }
        }
    }

    // 4. 保存更新后的 lastId
    await saveLastIdToFile(lastId, newEmails.length); // 保存处理完成后的 lastId

    // 5. 发送通知
    await sendNotification(dingtalk, {
        vendor: email.from,
        date: new Date().toLocaleString(),
        documentType: result.documentType,
        sn: result.invoiceNumber || result.receiptNumber,
        amount: result.amount
    });

    return '处理完成';
}

// 假设这些是您定义的函数
async function readLastIdFromFile(filePath) { ... }
async function fetchEmails(account, lastId) { ... }
async function attachFileToOCR(attachment, ocrUrl, ocrApiKey) { ... }
async function ocr(fileData) { ... } // 调用 OCR 服务并解析结果
async function submitInvoice(result, endpoint) { ... }
async function submitReceipt(result, endpoint) { ... }
async function saveLastIdToFile(filePath, newId) { ... }
async function sendNotification(endpoint, data) { ... }
