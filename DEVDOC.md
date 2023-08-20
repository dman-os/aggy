# news

## To-do

- [x] Comment counts
- [ ] Markdown input and html sanitization
- [ ] Stylization
  - [ ] Dark mode
- [ ] Some policy when the API is unable to contact other services
  - [ ] Attempt to recreate connections?
- [ ] Auth
  - [ ] Redis session cache
    - [ ] Last seen at on sessions
  - [ ] Expired token vacating cron job
  - [ ] Email verification
  - [ ] Password reset
  - [ ] 2FA
  - [ ] SSO
- [ ] Logging
- [ ] Replace UUIDs with HashIDs for user id?
- [ ] Use camel case error field codes in ValidationErrors

- [ ] Consider SurrealDb

- [x] Move to flywaydb for migrations

## design-doc

### Features

### Endpoints

#### Aggy

- [ ] User
  - [ ] Get
  - [ ] Create
  - [ ] Update
  - [ ] Delete
  - [ ] List

##### Epigram

- [ ] Epigram
  - [ ] Get
  - [ ] Create
  - [ ] Delete
  - [ ] List
- [ ] Submit Epigram
- [ ] Read Epigram
- [ ] Read children epigrams
- [ ] Confirm Epigram Email

##### Doface

## dev-log

### Upstream Issues

- [Postgres CITEXT support for SQLX](https://github.com/launchbadge/sqlx/issues/295)
- [envFile support for codelldb](https://github.com/vadimcn/vscode-lldb/issues/506)
- [$ref support for utoipa](https://github.com/juhaku/utoipa/issues/242)
