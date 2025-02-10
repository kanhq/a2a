# ChangeLog

## [v0.0.10] - 2025-02-10

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
