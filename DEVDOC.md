# news

## To-do

- [ ] FUll NIP-01 compliance
- [ ] Handle `{"limit":0}`
- [ ] Observability
- [x] Comment counts
- [ ] Markdown input and html sanitization
- [ ] Stylization
  - [ ] Dark mode
- [ ] Some policy when the API is unable to contact other services
  - [ ] Panic and kill the process
- [ ] Auth
  - [ ] Redis session cache
    - [ ] Last seen at on sessions
  - [ ] Expired token vacating cron job
  - [ ] Email verification
  - [ ] Password reset
  - [ ] 2FA
  - [ ] SSO
- [ ] Use camel case error field codes in ValidationErrors

- [ ] Consider SurrealDb
- [ ] Consider shuttle.rs

- [x] Move to flywaydb for migrations

## design-doc

### Features

#### Stretch goals

- [ ] Modlog
- [ ] Spec out APIs for epigram and doface

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
