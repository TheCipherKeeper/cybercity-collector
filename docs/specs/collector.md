# Модуль `collector`

## Назначение

Модуль объединяет прежние компоненты `ccc-core`, `ccc-host`, `ccc-telemetry`,
`ccc-kafka`, `ccc-command` и `ccc-node` без изменения runtime-поведения.

## Поведение

- Конфигурация загружается из TOML и переопределяется окружением.
- Telemetry читает разрешённые host-источники и формирует `AgentEvent`.
- Transport передаёт события и команды, command executor применяет policy.
- SIGINT/SIGTERM завершает фоновые задачи.

## Инварианты и ошибки

Наблюдение остаётся read-only; запрещённая команда увеличивает tamper-счётчик.
Ошибки конфигурации блокируют запуск, ошибки цикла журналируются.

## Проверка

| Поведение | Тест или пробная проверка |
|---|---|
| config и policy | тесты `core.rs` и `policy.rs` |
| защита пути host bridge | тесты `host.rs` |
| telemetry и transport | тесты `telemetry.rs` и `kafka.rs` |
| команды и lifecycle | тесты `command.rs` и `lifecycle.rs` |
