# ChangeLog

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
