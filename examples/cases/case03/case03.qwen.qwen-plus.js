// written by LLM provider: qwen model: qwen-plus
async function main(config, params) {
  // Read the last processed email ID from the save point file
  const lastIdResult = await doAction({
    kind: 'file',
    method: 'READ',
    path: config.email_save_point
  });
  const lastId = lastIdResult.id || 0;

  // Retrieve new emails
  const emails = await doAction({
    kind: 'email',
    method: 'RECV',
    account: config.email_account,
    last_id: lastId
  });

  let newLastId = lastId;
  const details = [];

  // Process each new email
  for (const email of emails) {
    // Update the last processed email ID if current email is newer
    if (email.id > newLastId) {
      newLastId = email.id;
    }

    // Check if the email is from tom@vendor.com and has attachments
    if (email.from === 'tom@vendor.com' && email.attachments.length > 0) {
      for (const attachmentPath of email.attachments) {
        // Read the attachment content
        const attachment = await doAction({
          kind: 'file',
          method: 'READ',
          path: attachmentPath
        });

        // Convert the attachment to base64 data URL
        const dataUrl = await doAction({
          kind: 'enc',
          methods: ['base64'],
          data: JSON.stringify(attachment)
        });

        // Perform OCR on the attachment
        const ocrResult = await doAction({
          kind: 'http',
          method: 'POST',
          url: config.ocr_url,
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${config.ocr_api_key}`
          },
          body: {
            image: `data:image/png;base64,${dataUrl}`
          }
        });

        // Process OCR result
        if (ocrResult.body && ocrResult.body.documentType) {
          const docType = ocrResult.body.documentType;
          const oaServiceUrl = docType === 'invoice' ? config.oa_invoice : 
                           docType === 'receipt' ? config.oa_receipt : null;

          if (oaServiceUrl) {
            // Submit the document to the appropriate OA service
            await doAction({
              kind: 'http',
              method: 'POST',
              url: oaServiceUrl,
              body: ocrResult.body
            });

            // Collect details for notification
            const vendor = email.from.split('@')[1];
            const date = new Date().toISOString().split('T')[0];
            const sn = docType === 'invoice' ? ocrResult.body.invoiceNumber : ocrResult.body.receiptNumber;
            const amount = ocrResult.body.amount;

            details.push({
              vendor,
              date,
              documentType: docType,
              sn,
              amount
            });
          }
        }
      }
    }
  }

  // Save the new last ID
  await doAction({
    kind: 'file',
    method: 'WRITE',
    path: config.email_save_point,
    body: { id: newLastId }
  });

  // Send notification via DingTalk bot if there are any processed documents
  if (details.length > 0) {
    let notificationContent = "# New Email Processing Completed\n\n";
    for (const detail of details) {
      notificationContent += `## Date: ${detail.date}\n\n`;
      notificationContent += `### Details\n\n`;
      notificationContent += `- Type: ${detail.documentType} No.: ${detail.sn} Amount: ${detail.amount}\n\n`;
    }

    await doAction({
      kind: 'notify',
      url: config.dingtalk,
      message: {
        msgtype: "text",
        text: {
          content: notificationContent
        }
      }
    });
  }

  return { processedEmails: emails.length, newLastId, details };
}
