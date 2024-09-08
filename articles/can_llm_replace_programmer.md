
大模型代替程序员来编写代码，这是一个非常有趣的话题，也是一个非常具有挑战性的任务，因为编写代码不仅仅是逻辑的表达，还涉及到了对环墫的依赖，对资源的管理，对错误的处理等等，这些都是大模型在生成代码时需要考虑的问题，目前，大模型作为编程的辅助工具，例如 Github Copilot, 或是当红的 Cursor, 可以在块级别，函数级别，甚至是文件级别生成代码，但基本上还是需要程序员的监督，对其生成的代码进行检查，修改，完善，才能真正的投入到生产环境中。

如果想让大模型产生代码生产可用，一个可能的方法是建立低代码工具，让大模型专注于逻辑的生成，而不用考虑环境的依赖，这与给人类使用的低代码工具是类似的，相对于人类，大模型不会抱怨编写低代码的无聊和繁琐。

# 测试方法

为了检测大模型的编程能力，我们通过让不同的大模型模型根据相同的 `prompt` 来生成代码，然后运行这些代码，检测代码的运行结果是否符合预期，这个过程会重复若干次，来检测其生成的稳定性。

我们使用了 `a2a` 这个专为大模型定制的低代码工具，它简化了对工作环境的依赖，封装了程序对数据库、文件、网络等资源的访问，使得大模型可以专注于逻辑代码的生成，而无需在环境搭建上花费更多的注意力，因为如果不限定环境，大模型生成的代码将非常的个性化，几乎不具备实际运行的可能。

# 测试逻辑

我们让大模型根据以下的逻辑 `prompt` 来生成代码，这是一段简单的逻辑，但是涉及到了数据库的操作，文件的读取，以及数据的处理，在阅读了其中的主要函数 `doAction` 的定义之后，人类编写这个逻辑的代码是非常简单的。

```
请使用配置中的 'dbconn' 连接数据库, 然后创建一个数据表 'test_users', 包含以下的字段

- id: 整数, 主键, 自增
- name: 字符
- age: 整数
- updated_at: 时间戳, 默认当前时间

从配置的 'datasrc' 数据文件连接读取数据，这是一个 CSV 文件, 包含两列 'name' 和 'age'

将读取到的数据插入到 'user' 表中

然后在数据表中查询 'age' 在 40 和 50 之间的记录, 作为最终的结果返回

最后，删除掉数据表 `test_users`
```

# 正确的代码

```javascript
// 本代码由 gpt-4o 生成

async function main(config, params) {
  // 创建测试表 'test_users'
  const createTableAction = {
    kind: "sql",
    connection: config.dbconn,
    query: `
      CREATE TABLE test_users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT,
        age INTEGER,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      )
    `,
  };

  await doAction(createTableAction);

  // 读取 CSV 文件中的数据
  const readFileAction = {
    kind: "file",
    method: "READ",
    path: config.datasrc,
  };

  const fileResult = await doAction(readFileAction);

  // 解析 CSV 数据并准备插入
  const insertRows = fileResult.map((row) => [row.name, row.age]);

  const insertDataAction = {
    kind: "sql",
    connection: config.dbconn,
    query: "INSERT INTO test_users (name, age) VALUES (?, ?)",
    rows: insertRows,
  };

  await doAction(insertDataAction);

  // 查询 'age' 在 40 和 50 之间的记录
  const queryDataAction = {
    kind: "sql",
    connection: config.dbconn,
    query: "SELECT * FROM test_users WHERE age BETWEEN 40 AND 50",
  };

  const result = await doAction(queryDataAction);

  // 删除测试表 'test_users'
  const dropTableAction = {
    kind: "sql",
    connection: config.dbconn,
    query: "DROP TABLE test_users",
  };

  await doAction(dropTableAction);

  // 返回查询结果
  return result;
}
```

# 测试结果

| 模型                                      | 总次数 | 成功次数 | 成功率 | 首 Token 时间 | 所有 Token 时间 |大概费用|
| ----------------------------------------- | ------ | -------- | ------ | ------------- | --------------- |-----|
| gpt-4o                                    | 14     | 14       | 100%   | 1019          | 4741            |¥0.108|
| anthropic.claude-3-5-sonnet-20240620-v1:0 | 14     | 14       | 100%   | 1260          | 11759           |¥0.087|
| glm-4-plus                                | 14     | 14       | 100%   | 538           | 19054           |¥0.200|
| gpt-4o-mini                               | 14     | 13       | 93%    | 611           | 6768            |¥0.004|
| qwen-plus                                 | 14     | 13       | 93%    | 716           | 19355           |¥0.012|
| qwen-max                                  | 14     | 13       | 93%    | 823           | 23837           |¥0.120|
| qwen-long                                 | 14     | 11       | 79%    | 848           | 16807           |
| gemini-1.5-pro                            | 14     | 10       | 71%    | 1447          | 9314            |¥0.076|
| gemini-1.5-flash                          | 14     | 9        | 64%    | 812           | 3103            |¥0.001|
| anthropic.claude-3-haiku-20240307-v1:0    | 14     | 8        | 57%    | 617           | 6794            |
| anthropic.claude-3-sonnet-20240229-v1:0   | 14     | 5        | 36%    | 1335          | 16807           |
| gemma2-9b-it                              | 14     | 3        | 21%    | 485           | 1277            |
| qwen-turbo                                | 14     | 3        | 21%    | 580           | 15418           |
| deepseek-coder                            | 14     | 3        | 21%    | 629           | 26133           |
| llama-3.1-70b-versatile                   | 14     | 2        | 14%    | 1502          | 3195            |
| Doubao-pro-32k                            | 14     | 1        | 7%     | 599           | 20205           |
| glm-4-flash                               | 14     | 1        | 7%     | 579           | 31949           |
| mixtral-8x7b-32768                        | 14     | 0        | 0%     | 448           | 1498            |
| Doubao-lite-32k                           | 14     | 0        | 0%     | 334           | 16185           |

# 结果分析

即便是对于这种几乎不依赖于环境，仅考察逻辑能力的代码生成任务，大模型的表现也是参差不齐，本次测试中，对大模型输入的 Token 大概是 1.5k 左右，代码产出的的 Token 大概是 0.5k 左右，所需费用以此来进行预估计算。

## 第一梯队

- `gpt-4o` 在 14 次测试中全部成功，平均每次生成代码的时间在 4.7秒左右，这是个非常好的表现。
- `anthropic.claude-3-5-sonnet-20240620-v1:0` 也是 14 次全部成功，平均每次生成代码的时间在 11.7秒，时间较长，但是成功率很高。
- `glm-4-plus` 也是 14 次全部成功，平均每次生成代码的时间在 19秒，时间较长，是国产模型中表现最好的。

## 第二梯队

- `gpt-4o-mini` 13 次成功，平均每次生成代码的时间在 6.7秒，成功率略低，但是速度快，而且 gpt-4o-mini 非常便宜，性价比很高。
- 通义千问家族的表现，除了 turbo 之外，成功率都在 90% 以上，也是国产模型中表现不错的。

## 第三梯队

- `gemini-1.5-flash` 9 次成功，平均每次生成代码的时间在 3.1秒，成功率较低，但是速度快，而且 gemini-1.5-flash 也是非常便宜的模型。可以考虑带着错误再次生成，整体的性价比还是很高的。
