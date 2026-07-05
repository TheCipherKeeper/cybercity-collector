# CyberCity — Collector

[![Part of CyberCity](https://img.shields.io/badge/CyberCity-composition-blueviolet)](https://github.com/TheCipherKeeper/cybercity)
[![License: MIT](https://img.shields.io/badge/code-MIT-green)](LICENSE)
[![Docs: CC BY 4.0](https://img.shields.io/badge/docs-CC%20BY%204.0-lightgrey)](LICENSE-DOCS)

Внешний наблюдатель за гостями цифрового двойника CyberCity. Работает на
хосте (гипервизор или K8s-нода), смотрит на гостей снаружи, только читает.
Подписывает наблюдения (Ed25519) и отправляет в `cybercity-engine` через
Kafka. Команды получает от `cybercity-manage`.

Ключевое свойство: коллектор недосягаем из range-сегмента — наблюдение
tamper-proof по конструкции.

## Статус

Каркас в переходе. Текущий код — in-guest MVP (tail логов, печать в stdout,
placeholder-подпись). Цель — out-of-band зонды, настоящая криптография,
реальный Kafka. Подробнее — в `docs/BACKLOG.md` и спеках crate'ов.

## Быстрый старт

```bash
cargo build
cp config/example.toml config/local.toml
cargo run --bin cybercity-collector -- config/local.toml
```

Коллектор читает логи из путей в конфиге и печатает события в stdout.

### Troubleshooting

**cargo run падает на чтении конфига** — проверить путь и TOML. `Config::load`
требует непустые `node_id` и `service_id` (или env `CCC_NODE_ID` / `CCC_SERVICE_ID`).

**Зонды не возвращают данные** — `ccc-telemetry` читает только пути из
`telemetry.log_paths`, разрешённые `policy.host_permissions`. Проверить, что
пути существуют и перечислены в `[[policy.host_permissions]]` с
`kind = "read_file"`.

### Конфигурация

TOML, грузится из первого аргумента CLI (default: `config/example.toml`).
Env-override через префикс `CCC_`:

| Поле | Env |
|---|---|
| node_id | CCC_NODE_ID |
| service_id | CCC_SERVICE_ID |
| kafka_broker | CCC_KAFKA_BROKER |

`policy.host_permissions` ограничивает, какие пути читать (`read_file`).
`policy.allowed_command_kinds` ограничивает kinds команд от manage.

## Документация

| Файл | Что |
|---|---|
| `AGENTS.md` | Правила работы: ветвление, коммиты, что можно/нельзя |
| `docs/INDEX.md` | Карта документации |
| `docs/ARCHITECTURE.md` | Архитектура: слои, потоки данных, доверительная граница |
| `docs/BACKLOG.md` | Очередь задач |
| `docs/specs/` | Контракты crate'ов (по одному файлу на crate) |

Архитектурные решения (ADR) — в хабе [`cybercity/adr/`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/).

## Разработка

```bash
git checkout dev
git pull
git checkout -b feat/<задача>

# внести изменения
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release

git commit -m "feat: ..."
git push
# открыть PR в dev
```

Полный цикл — в `AGENTS.md`. Задачи — в `docs/BACKLOG.md`.

## Лицензия

- Код: [MIT](LICENSE)
- Документация: [CC BY 4.0](LICENSE-DOCS)