# ChangeLog

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
