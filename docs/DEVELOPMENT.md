# Разработка

## Сборка и запуск

```bash
cargo build
cargo build --release

# Конфиг
cp config/example.toml config/local.toml
# отредактировать local.toml: node_id, service_id, kafka_broker, log_paths

# Запуск
cargo run --bin cybercity-collector -- config/local.toml
```

В текущем MVP коллектор читает логи из разрешённых путей и печатает события
в stdout (вместо Kafka — StdoutTransport). Реального брокера и Ed25519
подписи пока нет — см. спеки crate'ов, раздел «Что TODO».

## Конфигурация

TOML, грузится из первого аргумента CLI (default: `config/example.toml`).
Env-override через префикс `CCC_`:

| Поле | Env |
|---|---|
| node_id | CCC_NODE_ID |
| service_id | CCC_SERVICE_ID |
| kafka_broker | CCC_KAFKA_BROKER |

`policy.host_permissions` ограничивает, какие пути читать (`read_file`) и
какие service-units трогать (`exec_service`). `policy.allowed_command_kinds`
ограничивает kinds команд от manage.

## Тестирование

```bash
cargo test                          # все тесты workspace
cargo test -- --nocapture           # подробный вывод
cargo test -p ccc-host              # конкретный crate
```

Тесты — inline `#[cfg(test)]` модули внутри исходников. Полноценного
test-suite пока нет.

## Линтинг

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

CI пока не настроен.

## Troubleshooting

### cargo run падает на чтении конфига

Проверить путь и TOML. `Config::load` требует непустые `node_id` и
`service_id` (или env `CCC_NODE_ID` / `CCC_SERVICE_ID`).

### Зонды не возвращают данные

`ccc-telemetry` читает только пути из `telemetry.log_paths`, разрешённые
`policy.host_permissions`. Проверить, что пути существуют и перечислены в
`[[policy.host_permissions]]` с `kind = "read_file"`.