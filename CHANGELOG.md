# ChangeLog

## [v0.1.17] - 2025-05-27

### Improvement

- `http` action now supports timeout configuration.[service health check](examples/cases/case12/case12.vertex-ai.gemini-2.5-flash-preview-05-20.js)
- some minor bug fixes

## [v0.1.16] - 2025-05-20

### Add

- `file` action now write file as it's format, e.g. our can request llm to [convert 'data.json' to 'data.xlsx'](examples/cases/case11/case11.qwen.qwen-plus-latest.js)

### Improvement

- `shell` now use `cmd` on windows and `sh` on linux to run the command, which is more reliable

## [v0.1.15] - 2025-05-13

### Add

- more examples 
- more documentation
- `a2a init` command to initialize a work directory

## [v0.1.14] - 2025-04-01

### Add

- `a2a server` now supports [MCP](https://modelcontextprotocol.io/introduction)
  - a mcp tool 'a2a_run' can be used to run a2a code
  - a mcp prompt 'a2a' can be used to tell llm how to use a2a_run


# [v0.1.13] - 2025-03-14

### Enhancement

- 'web_search' and 'crawl' add anti bot detection

# [v0.1.12] - 2025-03-12

### Add

- `web_search` action used to search the web
- enable write code to search from the web to collect information, then write report by llm

# [v0.1.11] - 2025-03-10

### Fix

- some compile flags adjustments

## [v0.1.10] - 2025-02-10

### Add

- 'list' in file action now supports a wildcard pattern
- 'a2a run' now can cwd to a working directory

### Fix

- some minor bugs

### Modified

- use 'glob' instead of 'globwalk'

## [v0.1.9] - 2025-02-05

### Add

- new 'crawl' action used to crawl a website

### Modified

- share api definition with the code generation and nodejs binding

## [v0.1.8] - 2024-12-30

### Add

- log to file
- uuid_v7 function
- restore write code service
- restore list in file action

## [v0.1.7] - 2024-12-25

### Fix

- quickjs's `doAction` callback function now works more reliably
- `run` of service support file extraction

## [v0.1.6] - 2024-12-11

### Add

- `enc` action used to do encoding/encrypt

### Modified

## [v0.1.5] - 2024-11-26

### Modified

- bytes_to_json now detects the plain text and returns it as a string
- upgrade pyo3 to 0.23
- fs now support relative path

## [v0.1.4] - 2024-11-15

### Added

- `a2a server` can now exclude specific actions from being served
- `notify` action, used to send notifications to a user by IM service

## [v0.1.3] - 2024-11-01

### Added

- 'llm' action, used to invoke a Large Language Model (LLM) service

## [v0.1.2] - 2024-10-09

### Added

- 'schedule' stop/reload command
- 'shell' action

## [v0.1.1] - 2024-09-29

### Added

- `serve` command that host the generated code
  - export generated code as a web service to be consumed by other services
  - a scheduler to run the generated code at a specific time

## [v0.1.0] - 2024-09-20

- support actions
  - file: any file read/write operation
  - email: imap recv
  - http: http request
  - sql: sql query
