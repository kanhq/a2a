
[English](README.md) | [简体中文](README.cn.md)

# A2A: Enabling Large Models to Write More Correct Code

A2A (Api To Ai) is a set of advanced APIs and runtime designed to make it easier for large models to write more correct code and run it immediately without requiring additional installation or environment setup.

The goal of A2A is to enable non-professional developers to leverage large models to write correct code for their daily workflows and run it directly.

## Why A2A?

A2A aims to solve the following problems:
  - **Programming with Smaller Parameters Models**: For small parameters models (<=14B) that can be deployed locally, their programming capabilities are limited, and they might not be able to write correct code. Leveraging A2A's advanced APIs and runtime, these small models can more easily write correct code.
    - Deploying smaller-parameter large models locally (e.g., QWen3) provides better privacy protection and data security.
    - When using cloud-based large models, cheaper versions can be utilized for coding (e.g., Gemini Flash, GPT-4o-mini, Haiku), eliminating the need for more expensive models. These models are often faster and offer a better iterative experience.
  - **Code Correctness**: A2A provides a set of advanced APIs that encapsulate common functionalities, avoiding potential issues when large models write code:
      - When programming, large models are limited by context size, making errors more likely as code volume and detail increase.
      - Limited by their training data, large models' knowledge may be outdated, and the code they write might not run in new environments.
      - Some subtle handling might be overlooked by non-professional developers.
  - **Environment Setup**: The A2A model is "Download-Install-Run". As an "All in one" application, it requires no other environment configuration. Setting up most programming environments is difficult for non-professional developers:
      - It requires installing various dependencies, configuring environment variables, setting paths, etc.
      - Deploying as a service or for others to use can be even more troublesome.

## How A2A?

Refer to the instructions in [actions](bindings/nodejs/action.ts). A2A provides a function `doAction` to perform an action. Supported actions include:

  - `http`: Send HTTP requests, including GET/POST/PUT/...
  - `sql`: Execute SQL statements, including queries, inserts, updates, deletes, etc., for MySQL, PostgreSQL, SQLite
  - `file`: Read and write files including local, object storage services (S3/OSS...), and remote file systems(ftp...) (Thanks [Apache OpenDAL](https://github.com/apache/opendal))
  - `email`: Receive and send emails, IMAP
  - `shell`: Execute shell commands, automatically assembling command line parameters and processing output
  - `llm`: Call large models, for any model compatible with OpenAI API
  - `notify`: Send notifications, sending notifications to services that support Webhooks (e.g., DingTalk, Feishu, Slack)
  - `enc`: Encrypt and decrypt, supporting common encryption/encoding/hash algorithms
  - `crawl`: Crawl web content, use local browser for crawling
  - `web_search`: Search web content, using local browser to search and scrape search results

For a specific task, the large model will write business logic code based on the requirements, call these actions, and ultimately complete the task.

A2A embeds [quickjs](https://bellard.org/quickjs/) as its runtime, allowing the code written by the large model to be executed directly without other installations or configurations.

## Usage Modes

`A2A` has the following usage modes:

  - **Code Writing**: Have the large model write code based on user requirements.
  - **Code Running**: Run the code written by the large model.
  - **Service**: Provide functionalities such as code writing, running, scheduled tasks, static file hosting, and MCP in the form of an HTTP service.

### Code Writing

`a2a coder` is used for writing code. Users provide requirements, such as:

```markdown
Using the large model specified by conf.llm, obtain materials through web search, then write a research report on the usage of MCP (Model Context Protocol) in the large model field, and save the result as mcp.md.
```

The large model will write code based on the requirements [case10](examples/cases/case10/case10.vertex-ai.gemini-2.5-flash-preview-04-17.js).

Please check the examples in the [examples/cases](examples/cases) directory.

| No. | Category | Description |
|---|---|---|
|[case01](examples/cases/case01/case01.md)|Database|Read data from CSV, then batch write to database|
|[case02](examples/cases/case02/case02.md)|Database|Read data from JSON, then batch write to database|
|[case03](examples/cases/case03/case03.md)|Automation|Fetch and analyze new emails. If it is a supplier invoice, call OCR to extract information, then enter it into CRM, and send a notification|
|[case12](examples/cases/case12/case12.md)|Automation|Check the availability of a service and send a notification if it is unavailable.|
|[case13](examples/cases/case13/case13.md)|Automation|Use [AHK](https://www.autohotkey.com/) to do some UI Automation|
|[case04](examples/cases/case04/case04.md)|LLM Processing|Extract structured data from a given image|
|[case05](examples/cases/case05/case05.md)|File Processing|Extract specified pages from multiple PDFs and then merge them into a single PDF.|
|[case07](examples/cases/case07/case07.md)|File Processing|Find files with the specified name in the given directory and its subdirectories|
|[case09](examples/cases/case09/case09.md)|File Processing|Batch resize images and convert formats|
|[case11](examples/cases/case11/case11.md)|File Processing|Convert JSON to Excel|
|[case06](examples/cases/case06/case06.md)|Data Crawling|First get all fund links from the specified page, then extract detailed information of each fund as structured data|
|[case08](examples/cases/case08/case08.md)|Report Writing|Search the web to collect materials, then write a report|
|[case10](examples/cases/case10/case10.md)|Report Writing|Search the web to collect materials, then write a report|

### Code Running

`a2a run` is used to run the code written by the large model.

### Service

`a2a serve` starts the service. When starting, specify the working directory, where:

  - Files in the `api` directory store the code written by the large model and can be run via `POST /api/{filename}`.
  - Various configuration files are placed in the `conf` directory. They will be merged and passed as the `config` parameter when running the code.
  - Static files are placed in the `html` directory and can be accessed via `GET /{filename}`.
  - Scheduled tasks are placed in the `schedule` directory and will run at the specified time.

Additionally, `a2a serve` also provides the following paths:

  - `/mcp`: Provides service in the form of MCP, offering the tool `a2a_run` for running code written by large models, and a Prompt named `a2a` for writing code.
  - `POST /code`: Used for writing code.
  - `POST /run/json`: Used for running code, with the request body in JSON format.
  - `POST /run/form`: Used for running code, with the request body in form format.