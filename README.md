# CyberCity — Collector

[![Part of CyberCity](https://img.shields.io/badge/CyberCity-composition-blueviolet)](https://github.com/TheCipherKeeper/cybercity)
[![License: MIT](https://img.shields.io/badge/code-MIT-green)](LICENSE)
[![Docs: CC BY 4.0](https://img.shields.io/badge/docs-CC%20BY%204.0-lightgrey)](LICENSE-DOCS)

Внешний **out-of-band коллектор** цифрового двойника CyberCity. Работает на
хосте (гипервизор / K8s-нода), **снаружи** гостевых VM/контейнеров, read-only, в
mgmt-плоскости — и потому недосягаем из range-сегмента. Собирает телеметрию
зондами (файлы, сеть, память, процессы, сисколлы), подписывает события (Ed25519)
и шлёт в `cybercity-engine` по Kafka как **авторитетный** поток, на котором
считается scoring. Control-канал — от `cybercity-manage`.

> Канон состава, контрактов и доверительной границы —
> [`cybercity/COMPOSITION.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/COMPOSITION.md).

## Статус

**Каркас в переходе.** Текущий код — ещё in-guest MVP из эпохи
`cybercity-agents` (tail логов / stub-transport / placeholder-подпись).
Рефакторинг зондов в out-of-band, настоящая Ed25519-подпись и реальный
Kafka-transport (mTLS) — в дорожной карте (см. [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)).
Имена уже приведены к коллектору: crate'ы `ccna-*` → `ccc-*`, бинарник
`cybercity-node-agent` → `cybercity-collector`, env-префикс `CCNA_` → `CCC_`.

## Быстрый старт

```bash
cargo build
cp config/example.toml config/local.toml
cargo run --bin cybercity-collector -- config/local.toml
```

В текущем MVP коллектор читает логи из разрешённых `policy.host_permissions`
и печатает события в stdout (вместо Kafka — `StdoutTransport`).

## Документация

- [`AGENTS.md`](AGENTS.md) — governance: иерархия, принципы, правила для агента.
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — внутренняя архитектура (6-crate workspace).
- [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) — сборка, запуск, тестирование.
- [`docs/DATA_FLOW.md`](docs/DATA_FLOW.md) — поток данных: зонд → конверт → Kafka → engine.
- [`docs/adr/`](docs/adr/) — локальные архитектурные решения.

## Лицензия

- Код: [MIT](LICENSE)
- Документация: [CC BY 4.0](LICENSE-DOCS)