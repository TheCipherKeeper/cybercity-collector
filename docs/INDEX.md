# INDEX — карта документации

Точка входа для агентов и разработчиков. Прочитай этот файл первым,
потом грузи нужный спек.

## Порядок чтения

1. **INDEX.md** (этот файл) — карта
2. **ARCHITECTURE.md** — целевая архитектура: слои, потоки, trust boundary
3. **specs/ccc-*.md** — спек конкретного crate'а

## Файлы

| Файл | Что описывает |
|---|---|
| ARCHITECTURE.md | Целевая архитектура: 6 слоёв, 2 потока данных, trust boundary, mermaid |
| specs/ccc-core.md | Config, Policy, Lifecycle, AgentEvent, NodeIdentity, observation manifest |
| specs/ccc-host.md | HostBridge, read-only доступ, path traversal защита |
| specs/ccc-telemetry.md | Цикл сбора, чтение manifest, мультиплексирование источников |
| specs/ccc-kafka.md | Transport trait, SecureTransport, подпись, топики |
| specs/ccc-command.md | CommandExecutor, проверка policy, tamper detection |
| specs/ccc-node.md | Composition root, 3 tokio-таски, graceful shutdown |

## Правила для агентов

- Прочитай ARCHITECTURE.md перед работой с любым crate'ом.
- Прочитай спек crate'а перед изменением кода в нём.
- После реализации — обнови спек: перенеси пункт из "Что TODO" в "Что есть".
- Создавай feature-ветку от `dev`: `feat/<task>` или `fix/<task>`.
- Коммить в feature-ветку, открывай PR в `dev` (не в `main`).
- Прямой коммит в `main` или `dev` — запрещён. Только через feature-ветку + PR.
- `main` — стабильная, вливается только из `dev`. `dev` — разработческая.
- Сообщения коммитов — Conventional Commits (см. AGENTS.md, docs/DEVELOPMENT.md).
- ADR живут в хабе `cybercity/adr/`, не здесь.
- Вся документация — русский. Код-идентификаторы — английский.

## Связь с AGENTS.md

AGENTS.md — governance (правила работы, что можно/нельзя).
docs/ — спецификация (что должно быть реализовано).
Код — реализация (как сделано).