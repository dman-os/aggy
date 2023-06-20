# news

## To-do

- [ ] Move to flywaydb for migrations
- [ ] Some policy when the API is unable to contact other services
  - [ ] Attempt to recreate connections?
- [ ] Auth
  - [ ] Redis session cache
  - [ ] Expired token vacating cron job
  - [ ] Email verification
  - [ ] Password reset
  - [ ] 2FA
  - [ ] SSO
- [ ] Logging
- [ ] Replace UUIDs with HashIDs for user id

## design-doc

### Features

### Endpoints

- [ ] User
  - [ ] Get
  - [ ] Create
  - [ ] Update
  - [ ] Delete
  - [ ] List

## dev-log

### Upstream Issues

- [Postgres CITEXT support for SQLX](https://github.com/launchbadge/sqlx/issues/295)
- [envFile support for codelldb](https://github.com/vadimcn/vscode-lldb/issues/506)
- [$ref support for utoipa](https://github.com/juhaku/utoipa/issues/242)
