# AGENTS.md — сервис CyberCity Collector

Репозиторий содержит один независимо поставляемый сервис. Общие полномочия,
рабочий цикл, архитектурный канон и поставка заданы методологией
`TheCipherKeeper/ai-project-template` на версии из `.methodology.yml`.
Системные границы и общие контракты находятся в хабе
`TheCipherKeeper/cybercity`.

## Локальные правила

- Стек сервиса — только Rust.
- Модуль `collector` следует границам `domain`, `application`, `ports`,
  `adapters`; прежние crate `ccc-*` сохранены как внутренние Rust-модули.
- Наблюдение целей read-only; reset и изоляция принадлежат manage.
- События и команды передаются через брокер; прямые межсервисные вызовы
  запрещены.
- Код, тесты, архитектура и спецификация обновляются согласованно.

## Команды

| Стек | Проверка | Тест | Сборка |
|---|---|---|---|
| Rust | `cargo fmt --check && cargo clippy -- -D warnings` | `cargo test --locked` | `cargo build --release --locked` |
