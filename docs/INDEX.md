# docs/INDEX — карта документации

Точка входа. Прочитай этот файл первым, потом грузи нужный спек.

## Порядок чтения

1. `AGENTS.md` — правила работы в репозитории.
2. `INDEX.md` (этот файл) — карта.
3. `BACKLOG.md` — очередь задач.
4. `ARCHITECTURE.md` — архитектура системы.
5. `specs/<crate>.md` — контракт нужного crate'а.

## Файлы

| Файл | Что описывает |
|---|---|
| AGENTS.md | Правила: ветвление, что можно/нельзя, коммиты, ADR |
| BACKLOG.md | Очередь задач с приоритетами и зависимостями |
| ARCHITECTURE.md | Система: 6 слоёв, 2 потока данных, доверительная граница |
| specs/ccc-core.md | Config, Policy, Lifecycle, AgentEvent, observation manifest |
| specs/ccc-host.md | HostBridge, read-only доступ, защита от path traversal |
| specs/ccc-telemetry.md | Цикл сбора, manifest, мультиплексирование источников |
| specs/ccc-kafka.md | Transport, SecureTransport, подпись, топики |
| specs/ccc-command.md | CommandExecutor, проверка policy, tamper detection |
| specs/ccc-node.md | Точка сборки, 3 tokio-таски, graceful shutdown |
| DEVELOPMENT.md | Сборка, запуск, тестирование, troubleshooting |

## Как работать со спеками

- Прочитай `ARCHITECTURE.md` перед работой с любым crate'ом.
- Прочитай спек crate'а перед изменением кода в нём.
- После реализации — обнови спек: перенеси пункт из «Что TODO» в «Что есть».
- Каждый пункт «Что есть» в спеке → должен иметь тест.
- Каждый спек следует структуре: описание / Интерфейсы / Типы / Что есть /
  Что TODO / Ограничения / Зависимости.
- Спек описывает контракт (что), не реализацию (как).

## Шлюз качества

`scripts/verify.sh` — единая команда: fmt + clippy + test + build.
Запусти перед коммитом. Если fail — чини, не коммить.
CI (GitHub Actions) прогоняет тот же скрипт на каждый PR в dev.
PR не вливается если verify не прошёл.