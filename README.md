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

## Документация

| Файл | Что |
|---|---|
| `AGENTS.md` | Правила работы: ветвление, коммиты, что можно/нельзя |
| `docs/INDEX.md` | Карта документации |
| `docs/ARCHITECTURE.md` | Архитектура: слои, потоки данных, доверительная граница |
| `docs/BACKLOG.md` | Очередь задач |
| `docs/specs/` | Контракты crate'ов (по одному файлу на crate) |
| `docs/DEVELOPMENT.md` | Сборка, тесты, troubleshooting |

Архитектурные решения (ADR) — в хабе [`cybercity/adr/`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/).

## Разработка

```bash
git checkout dev
git pull
git checkout -b feat/<задача>

# внести изменения
./scripts/verify.sh    # fmt + clippy + test + build
git commit -m "feat: ..."
git push
# открыть PR в dev
```

Полный цикл — в `AGENTS.md`. Задачи — в `docs/BACKLOG.md`.

## Лицензия

- Код: [MIT](LICENSE)
- Документация: [CC BY 4.0](LICENSE-DOCS)