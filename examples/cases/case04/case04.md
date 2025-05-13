调用 `llm01` 配置的大模型， 让它对参数中 `file` 代表的图片进行识别，并让它用 `JSON` 格式返回结果
返回的 JSON 中，必须符合以下的结构

```typescript
type Result = {
  name?: string;
  address?: string;
  phone?: string;
  email?: string;
};
```
