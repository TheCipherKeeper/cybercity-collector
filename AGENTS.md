# AGENTS.md — правила для AI-агентов и контрибьюторов CyberCity Collector

## Иерархия документов (от старшего к младшему)

**Над репозиторием** — хаб `cybercity` держит системные документы:

- [`cybercity/COMPOSITION.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/COMPOSITION.md) — канон состава, контрактов, доверительной границы.
- [`cybercity/CONVENTIONS.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/CONVENTIONS.md) — кросс-репо конвенции (язык, скелет репо, ADR-формат, event envelope).
- [`cybercity/adr/`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/) — сквозные ADR (почему 6 репо, доверительная граница, Rust-коллектор).

**Внутри репозитория:**

1. `AGENTS.md` (этот файл) — операционные правила работы в репозитории.
2. `README.md` — краткое описание и quick start.
3. `docs/` — внутренняя документация (`ARCHITECTURE.md`, `DEVELOPMENT.md`).
4. Код, тесты, конфиги — реализация принятых решений.

Если документы противоречат друг другу, побеждает старший. Любое расхождение —
повод создать новый ADR в хабе (см. ниже «Где живут ADR»).

## Ключевые принципы

- **Out-of-band, не in-guest.** Наблюдатель живёт на привилегию выше цели
  (гипервизор / K8s-нода) и недосягаем из range-сегмента — tamper-proof по
  конструкции, а не по усилию. Текущий код — in-guest MVP; целевое состояние —
  out-of-band зонды (см.
  [`cybercity/adr/0003-collector-rust-out-of-band.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/0003-collector-rust-out-of-band.md)).
- **Read-only.** Коллектор не пишет в гости; действия (reset/изоляция) — через
  `cybercity-manage`/фабрику, не через in-guest код.
- **Подписанные события (Ed25519).** Каждый конверт подписан (key id + nonce +
  timestamp, replay-protection); `engine` верифицирует. Kafka — mTLS + ACL на
  продюсеров. Сейчас — placeholder; реализация в дорожной карте.
- **In-guest никогда не доверен.** Опциональное in-guest обогащение —
  best-effort, **никогда** не источник для scoring. «Агент замолчал» — сам по
  себе сигнал.
- **Target-агностичен: все runtime-виды единообразно.** Коллектор наблюдает
  `{vm, container, lite}` (umbrella ADR-0004) одним probe-pipeline; `honeypot` —
  purpose-флаг от manage, не отдельный режим. Класса «engine-synthesized service
  events» нет — движок регистратор, не симулятор (всё, что движок знает о цели,
  приходит подписанным наблюдением).
- **Crate-имена `ccc-*`.** Старые `ccna-*` / `cybercity-node-agent` сняты;
  бинарник — `cybercity-collector`; env-префикс — `CCC_`.

## Правила для AI-агентов

### Что агенту МОЖНО

- Писать Rust-код в `crates/` (новые зонды, transport, command-handlers).
- Редактировать `Cargo.toml` зависимости с обоснованием.
- Запускать `cargo build`, `cargo run --bin cybercity-collector`, `cargo test`,
  `cargo fmt --check`, `cargo clippy -- -D warnings`.
- Обновлять `README.md`, `AGENTS.md`, `docs/` при изменении структуры.
- Создавать feature-ветки от `dev`, коммитить в них, пушить, открывать PR в `dev`.
- Обновлять спеки в `docs/` (переносить пункты из "Что TODO" в "Что есть"
  после реализации).

### Чего агенту НЕЛЬЗЯ

- Создавать ADR внутри этого репозитория: все ADR живут только в хабе
  `cybercity/adr/` (см. ADR-0005 и
  [`cybercity/adr/README.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/README.md)).
  Новое архитектурное решение фиксируется в хабе, а не в `docs/`.
- Редактировать ADR хаба без явного указания или создания нового ADR.
- Добавлять зависимости без обоснования в ADR хаба или комментарии.
- Коммитить напрямую в `main` или `dev` — только через feature-ветку + PR.
- Пушить в `main` — main обновляется только через merge из `dev`.
- Фабриковать реализацию out-of-band зондов или Ed25519-подписи, которой нет в
  коде: описывать честно — что есть stub/placeholder, что — TODO.
- Трогать `Cargo.lock`, `config/`, `fixtures/`, `target/`, `.gitignore` без
  явного одобрения.

## Структура репозитория

```
cybercity-collector/
├── README.md                         # обзор + quick start + contributing-указатель
├── AGENTS.md                         # этот файл
├── Cargo.toml                        # workspace (6 членов)
├── Cargo.lock                        # pinned deps
├── LICENSE                           # MIT
├── LICENSE-DOCS                      # CC BY 4.0
├── config/
│   └── example.toml                   # пример конфига → копируют в local.toml
├── crates/                           # workspace-члены
│   ├── ccc-core/                     # runtime, config, lifecycle, policy, envelope-типы
│   ├── ccc-host/                      # read-only доступ к хосту (target: out-of-band зонды)
│   ├── ccc-telemetry/                 # сбор событий из зондов
│   ├── ccc-kafka/                     # transport (stub/Stdout) + подпись конверта (placeholder)
│   ├── ccc-command/                   # control-команды от manage
│   └── ccc-node/                      # бинарник cybercity-collector (composition root)
├── fixtures/                         # тестовые данные логов
└── docs/                             # документация
    ├── INDEX.md                      # точка входа: карта + правила для агентов
    ├── ARCHITECTURE.md               # целевая архитектура: слои, потоки, trust boundary
    ├── DEVELOPMENT.md                # сборка, запуск, тестирование
    └── specs/                        # спеки crate'ов (контракты)
        ├── ccc-core.md               # Config, Policy, Lifecycle, AgentEvent
        ├── ccc-host.md               # HostBridge, read-only, path traversal
        ├── ccc-telemetry.md          # цикл сбора, manifest, мультиплексирование
        ├── ccc-kafka.md             # Transport, SecureTransport, подпись, топики
        ├── ccc-command.md            # CommandExecutor, policy, tamper
        └── ccc-node.md               # composition root, 3 таски, shutdown
```

## Рабочий цикл

1. Прочитать `docs/INDEX.md` → `docs/ARCHITECTURE.md` → спек нужного crate'а.
2. Создать feature-ветку от `dev`: `git checkout dev && git pull && git checkout -b feat/<task-name>`.
3. Внести изменения в коде.
4. Запустить `cargo build`, `cargo test`, `cargo fmt --check`,
   `cargo clippy -- -D warnings`.
5. Обновить спек: перенести реализованные пункты из "Что TODO" в "Что есть".
6. Закоммитить с conventional commit сообщением (см. `docs/DEVELOPMENT.md`).
7. Запушить feature-ветку.
8. Открыть PR в `dev` (не в `main`).
9. После review и merge в `dev` — ветка удаляется.

## Модель ветвления

```
main (стабильная)
  ▲
  │ merge (только из dev, человек)
  │
dev (разработка)
  ▲
  │ PR + merge (из feature-веток)
  │
├── feat/manifest-types
├── feat/ed25519-signing
├── feat/incremental-tail
└── fix/path-traversal-edge
```

- `main` — стабильная, релизная. Вливается только из `dev`.
- `dev` — разработческая. Вливается из feature-веток через PR.
- feature-ветки — от `dev`, вливаются в `dev` через PR, удаляются после merge.
- Прямой коммит в `main` или `dev` — запрещён. Только через feature-ветку + PR.

## Где живут ADR

Все архитектурные решения (ADR) живут **только в хабе** `cybercity/adr/` —
сквозные по всем репозиториям. В этом репозитории `docs/adr/` не ведётся
(см. hub
[`adr/0005-adr-centralized-in-hub.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/0005-adr-centralized-in-hub.md)).
Индекс решений —
[`cybercity/adr/README.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/README.md).
Формат ADR — в
[`cybercity/CONVENTIONS.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/CONVENTIONS.md).

## Язык документации

Вся документация и ADR ведутся на русском языке. README может содержать
английские бейджи и ссылки, но основной текст — русский. Английский допустим
только для: бейджей, идентификаторов кода, имён crate'ов, значений поля
`Status:` ADR (`Accepted` / `Superseded` / `Amended`).