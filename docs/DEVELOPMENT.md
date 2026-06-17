# CyberCity Collector — Руководство разработчика

## Быстрый старт

```bash
cd /path/to/cybercity-collector

# Сборка
cargo build
cargo build --release

# Конфиг из примера
cp config/example.toml config/local.toml
# отредактировать local.toml: node_id, service_id, kafka_broker, log_paths

# Запуск
cargo run --bin cybercity-collector -- config/local.toml
```

В текущем MVP коллектор читает логи из разрешённых `policy.host_permissions`
и печатает события в stdout (вместо Kafka — `StdoutTransport`). Реального
брокера и Ed25519-подписи пока нет — это TODO (см.
[`ARCHITECTURE.md`](ARCHITECTURE.md)).

## Структура проекта

```
cybercity-collector/
├── Cargo.toml                       # workspace (6 членов)
├── Cargo.lock                        # pinned deps
├── config/
│   └── example.toml                  # пример конфига → копируют в local.toml
├── crates/                           # workspace-члены
│   ├── ccc-core/                     # config, lifecycle, policy, envelope-типы
│   ├── ccc-host/                     # read-only host-доступ
│   ├── ccc-telemetry/                # сбор событий из зондов
│   ├── ccc-kafka/                     # transport (stub) + подпись (placeholder)
│   ├── ccc-command/                  # control-команды от manage
│   └── ccc-node/                     # бинарник cybercity-collector
├── fixtures/                         # тестовые данные логов
└── docs/                             # документация
    ├── ARCHITECTURE.md
    ├── DEVELOPMENT.md
    ├── DATA_FLOW.md
    └── adr/
```

## Конфигурация

Конфиг — TOML, грузится из пути первого аргумента CLI (по умолчанию
`config/example.toml`). Переопределения через env с префиксом `CCC_`:

| Поле | Env |
|---|---|
| `node_id` | `CCC_NODE_ID` |
| `service_id` | `CCC_SERVICE_ID` |
| `kafka_broker` | `CCC_KAFKA_BROKER` |

`policy.host_permissions` ограничивает, какие пути можно читать (`read_file`)
и какие service-units трогать (`exec_service`). `policy.allowed_command_kinds`
ограничивает kinds команд от `cybercity-manage`.

## Работа над коллектором

### Добавление нового зонда (цель — out-of-band)

1. Определить модуль зонда в `crates/ccc-host/` (или отдельный crate при
   росте).
2. Реализовать read-only сбор через `HostBridge`/привилегированный доступ на
   гипервизоре (ZFS-snapshot, eBPF, `/proc`-walk и т.п.).
3. Пропустить наблюдения через `ccc-telemetry` → `AgentEvent`.
4. Добавить политику в `config/example.toml`.

> Сейчас зонды — только tail логов; настоящие out-of-band зонды — TODO.

### Добавление нового типа события

1. Расширить конверт/envelope-типы в `ccc-core` (цель — канонический event
   envelope из `cybercity/CONVENTIONS.md`).
2. Добавить сбор в `ccc-telemetry`.
3. Добавить тесты (см. «Тестирование»).

### Подключение реального Kafka-transport

1. В `crates/ccc-kafka/Cargo.toml` включить feature `real`
   (`rdkafka` + `rustls` — уже закомментированы как заготовка).
2. Реализовать `Transport` для реального брокера (mTLS/SASL).
3. Реализовать Ed25519-подпись в `SecureTransport` (key id + nonce + timestamp).
4. domain-логика в `ccc-core`/`ccc-telemetry` при этом не меняется.

## Тестирование

Тесты сейчас — **inline** `#[cfg(test)]` модули внутри исходников (см.
`crates/ccc-host/src/lib.rs`: `mod tests` с `#[tokio::test]`). Полноценного
тестового набора пока нет.

```bash
# Запустить все тесты workspace
cargo test

# С подробным выводом
cargo test -- --nocapture

# Конкретный crate
cargo test -p ccc-host
```

> **TODO:** полноценный test-suite (unit-тесты per-crate, интеграционные тесты
> на transport/команды, фикстуры логов в `fixtures/`). Сейчас тесты есть только
> в `ccc-host`.

## Линтинг и проверки

Линтеры и форматирование **пока не настроены** в CI. Рекомендуемые команды
(TODO — добавить в workflow):

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

## Стиль коммитов

Conventional Commits (см.
[`cybercity/CONVENTIONS.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/CONVENTIONS.md)):

```text
feat: add zfs-snapshot fs probe
fix: reject path traversal in HostBridge
docs: update ARCHITECTURE.md with probe matrix
refactor: extract envelope signing into SecureTransport
adr: record out-of-band decision as ADR-0001
```

Breaking changes включают `BREAKING CHANGE:` в тело. Summary line — английский
допустим; тело коммита — на русском.

## Процесс ADR

Если изменение затрагивает архитектурное решение:

1. Написать или обновить ADR в `docs/adr/`.
2. Сослаться на него из `docs/ARCHITECTURE.md`.
3. Старые ADR помечать `Superseded`, а не удалять.

Формат ADR — в
[`cybercity/CONVENTIONS.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/CONVENTIONS.md).

## Troubleshooting

### `cargo run` падает на чтении конфига

Убедитесь, что путь существует и TOML валиден:

```bash
cargo run --bin cybercity-collector -- config/local.toml
```

`Config::load` требует непустые `node_id` и `service_id` (или env `CCC_NODE_ID`
/ `CCC_SERVICE_ID`).

### Зонды не возвращают данные

В текущем MVP `ccc-telemetry` читает только пути из `telemetry.log_paths`,
разрешённые `policy.host_permissions`. Проверьте, что пути существуют и
перечислены в `[[policy.host_permissions]]` с `kind = "read_file"`.

## Связанные документы

- [`AGENTS.md`](../AGENTS.md) — правила для AI-агентов.
- [`docs/ARCHITECTURE.md`](ARCHITECTURE.md) — внутренняя архитектура.
- [`docs/DATA_FLOW.md`](DATA_FLOW.md) — поток данных.
- [`docs/adr/`](adr/) — локальные ADR.