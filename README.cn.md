[English](README.md) | [简体中文](README.cn.md)

# A2A: 让大模型编写更正确的代码

A2A (Api To Ai) 是一套高级API和运行时，以便于让大模型编写出更正确的代码，然后立即运行它，而无需更多的安装、环境搭建。

A2A 的目标是让非专业的开发者能够利用大模型，为他的日常工作流，编写出正确的代码，然后直接运行。

## 为何 A2A?

A2A 旨在解决以下问题：

- **小参数模型编程**：对于可以部署在本地的小参数模型（<=14B），它们的编程能力有限，可能无法编写出正确的代码。基于A2A 的高级API和运行时，这些小模型可以更容易地编写出正确的代码。
  - 在本地部署小参数大模型时(QWen3等)，会提供更好的隐私保护和数据安全性。
  - 在使用云端大模型时, 则可以使用更便宜的版本来编写代码(Gemini Flash, GPT-4o-mini, HaiKu等)，而不需要使用更昂贵的模型, 这些模型通常更加快速, 也会提供更好的迭代体验.
- **代码正确性**：A2A 提供了一套高级API，封装了常用的功能，避免了大模型编写代码时，可能出现的：
  - 大模型在编程时，受限于上下文大小，如果代码量增加，细节增加，容易出现错误。
  - 受限于其训练数据，大模型的知识可能过时，其编写的代码可能无法运行在新的环境中。
  - 一些微妙的处理，对于非专业的开发者来说，可能会被忽略。
- **环境搭建**：A2A 的模式是 `下载-安装-运行`, 作为一个 `All in one` 的应用, 无需其他的环境配置。而大多数编程环境的搭建，对于非专业的开发者来说，是一件困难的事情：
  - 需要安装各种依赖，配置环境变量，设置路径等。
  - 部署成服务，或给他人使用，可能更加麻烦。

## 如何 A2A?

参考 [actions](bindings/nodejs/action.ts) 的说明, A2A 提供了一个函数 `doAction`，用于执行一个动作，支持的动作包括

- `http`: 发送http请求, GET/POST/PUT/...
- `sql`: 执行sql语句, 包括查询、插入、更新、删除等, MySQL、PostgreSQL、SQLite
- `file`: 读写包括本地、对象存储服务(S3/OSS...)、远程文件系统(ftp...)的文件（Thanks [Apache OpenDAL](https://github.com/apache/opendal)）
- `email`: 收取和发送邮件, IMAP
- `shell`: 执行shell命令, 会自动组装命令行参数, 并处理输出
- `llm`: 调用大模型, 调用任何兼容 OpenAI API 的大模型
- `notify`: 发送通知, 向支持Webhook的服务发送通知(如钉钉、飞书、Slack等)
- `enc`: 加密和解密, 常用的加解密/编码/哈希算法
- `crawl`: 爬取web内容, 调用本地浏览器进行爬取
- `web_search`: 搜索web内容, 通过本地浏览器进行搜索, 并抓取搜索结果

对于一个具体的任务, 大模型会根据需求, 编写业务逻辑代码, 调用这些动作, 最终来完成任务. 

A2A 嵌入了 [quickjs](https://bellard.org/quickjs/) 作为运行时, 这使得大模型编写的代码可以直接执行, 而无需其他的安装和配置.

## 使用模式

`A2A` 有如下的几种使用模式

- **编写代码**：让大模型根据用户的需求，编写代码
- **运行代码**：运行大模型编写的代码
- **服务**：以HTTP服务的形式提供包括代码编写、运行、定时任务, 静态文件托管, MCP等功能

### 代码编写

`a2a coder` 用于编写代码, 用户提供需求, 如
```markdown
用 conf.llm 指定的大模型，从网络搜索获取素材，然后编写一份关于MCP(Model Context Protocol)在大模型领域使用情况的的研究报告，保存结果为 mcp.md
```

大模型会根据需求, [编写代码](examples/cases/case10/case10.vertex-ai.gemini-2.5-flash-preview-04-17.js)

请查看 [examples/cases](examples/cases) 目录下的示例

|编号|分类|说明|
|---|---|---|
|[case01](examples/cases/case01/case01.cn.md)|数据库|从CSV中读取数据,然后批量写入数据库|
|[case02](examples/cases/case02/case02.md)|数据库|从JSON中读取数据,然后批量写入数据库|
|[case03](examples/cases/case03/case03.cn.md)|自动化|收取分析新邮件, 当它是供货商发票时, 调用OCR抽取信息, 然后录入到CRM中, 然后发送通知|
|[case12](examples/cases/case12/case12.cn.md)|自动化|检查服务的可用性，不可用时发送通知|
|[case13](examples/cases/case13/case13.cn.md)|自动化|使用[AHK](https://www.autohotkey.com/)进行一些UI自动化|
|[case04](examples/cases/case04/case04.md)|大模型处理|从给定的图片中, 提取结构化数据|
|[case05](examples/cases/case05/case05.cn.md)|文件操作|从多个PDF中, 抽取指定页, 然后合并到一个PDF中|
|[case07](examples/cases/case07/case07.md)|文件操作|在给定的目录及其子目录中, 查找指定名称的文件|
|[case09](examples/cases/case09/case09.cn.md)|文件操作|批量将图片更改大小和转换格式|
|[case11](examples/cases/case11/case11.md)|文件操作|转换JSON为Excel|
|[case06](examples/cases/case06/case06.cn.md)|数据抓取|首先获取指定页面中的所有基金链接, 然后提取每个基金的详细信息为结构化数据|
|[case08](examples/cases/case08/case08.md)|报告编写|搜索网络收集素材, 然后编写报告|


### 代码运行

`a2a run` 用于运行大模型编写的代码

### 服务

`a2a serve` 启动服务, 启动时指定工作目录, 其中

- `api` 目录下的文件, 放置大模型编写的代码, 可以通过 `POST /api/{filename}` 来运行
- `conf` 目录下放置各种配置文件, 它们会合并在一起, 在运行代码时, 作为 `config` 参数传入
- `html` 目录下放置静态文件, 可以通过 `GET /{filename}` 来访问
- `schedule` 目录下放置定时任务, 这些任务会在指定的时间运行

额外的, `a2a serve` 还提供以下的路径

- `/mcp` 以MCP的形式提供服务, 提供了工具 `a2a_run` 用于运行大模型编写的代码, 以及名为 `a2a` 的 Prompt, 用于编写代码
- `POST /code` 用于编写代码
- `POST /run/json` 用于运行代码, 请求体为json格式
- `POST /run/form` 用于运行代码, 请求体为form格式