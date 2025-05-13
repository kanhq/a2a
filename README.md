
[English](README.md) | [简体中文](README.cn.md)

# A2A: Enabling Large Models to Write More Correct Code

A2A (Api To Ai) is a set of advanced APIs and runtime designed to make it easier for large models to write more correct code and run it immediately without requiring additional installation or environment setup.

The goal of A2A is to enable non-professional developers to leverage large models to write correct code for their daily workflows and run it directly.

## Why A2A?

A2A aims to solve the following problems:

  - **Code Correctness**: A2A provides a set of advanced APIs that encapsulate common functionalities, avoiding potential issues when large models write code:
      - When programming, large models are limited by context size, making errors more likely as code volume and detail increase.
      - Limited by their training data, large models' knowledge may be outdated, and the code they write might not run in new environments.
      - Some subtle handling might be overlooked by non-professional developers.
  - **Environment Setup**: The A2A model is "Download-Install-Run". As an "All in one" application, it requires no other environment configuration. Setting up most programming environments is difficult for non-professional developers:
      - It requires installing various dependencies, configuring environment variables, setting paths, etc.
      - Deploying as a service or for others to use can be even more troublesome.

## How A2A?

Refer to the instructions in [actions](bindings/nodejs/action.ts). A2A provides a function `doAction` to perform an action. Supported actions include:

  - http: Send HTTP requests
  - sql: Execute SQL statements
  - file: Read and write files including local, object storage services, and remote file systems (Thanks [Apache OpenDAL](https://github.com/apache/opendal))
  - email: Receive and send emails
  - shell: Execute shell commands
  - llm: Call large models
  - notify: Send notifications
  - enc: Encrypt and decrypt
  - crawl: Crawl web content
  - web_search: Search web content

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