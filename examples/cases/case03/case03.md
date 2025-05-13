## 检查新邮件

- 从 'email_save_point' 配置的 JSON 文件中，读取 'lastId' 的值。请注意，它是一个 OSS 文件。
- 从 'email_account' 配置的账户，获取新邮件。
- 对获取到的文件，如果其发件人是 'tom@vendor.com', 并且有附件，则将附件中的图片或 PDF 提交给 `OCR` 服务。

## 对邮件中的附件进行 OCR

`OCR` 服务是一个 HTTP restful API，其请求参数如下：

- method: 'POST'
- url: 从配置 'ocr_url' 中获取
- headers: 包含 JSON 格式的 Content-Type, 以及从配置 'ocr_api_key' 中获取的 Authorization
- body: {
  image: '文件的的 DataURL 表示',
  }

## 保存新邮件的 ID

保存 'lastId' 的值到 'email_save_point' 配置的 JSON 文件中。

## 处理结果

- 如果 'OCR' 服务返回的结果中包含的 `documentType` 是 'invoice', 调用配置中的 `oa_invoice` 指定的 OA 服务，提交该发票。
- 如果 'OCR' 服务返回的结果中包含的 `documentType` 是 'receipt', 调用配置中的 `oa_receipt` 指定的 OA 服务，提交该收据。

## 发送通知

处理完成后, 调用配置中 `dingtalk` 指定的钉钉机器人，发送通知。通知的内容根据以下模板进行填充

```markdown
# {vendor} 新邮件处理完成

# 日期: {date}

# 明细

- 类型: {documentType} 票号: {sn} 金额: {amount}
```

其中

- `{vendor}` 为邮件中的发件人域名
- `{date}` 为当前日期
- `{documentType}` 为 `OCR` 服务返回的结果中的 `documentType`
- `{sn}` 为 `OCR` 服务返回的结果中的 `invoiceNumber` 或 `receiptNumber`
- `{amount}` 为 `OCR` 服务返回的结果中的 `amount`
- 明细可能有多条，每条对应一个 `OCR` 服务返回的结果
