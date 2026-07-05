# ccc-kafka

## Что это

Transport-слой и доверительная граница. Подпись конверта (Ed25519), отправка
в Kafka, приём команд. Точка, где наблюдения пересекают доверительную
границу — подпись утверждает авторство коллектора.

## Интерфейсы

- `trait Transport` — `send_event(topic, event)` / `receive_command()`.
  Абстракция над брокером, чтобы коллектор работал без реального Kafka.
- `StdoutTransport` — stub, печатает события в stdout.
- `SecureTransport<T: Transport>` — обёртка с подписью (placeholder).
- `TopicNames::from_config(cfg)` — генерация имён топиков из service_id.

## Типы

```rust
pub trait Transport: Send + Sync {
    async fn send_event(&self, topic: &str, event: &AgentEvent) -> Result<(), KafkaError>;
    async fn receive_command(&mut self) -> Result<Option<CommandEnvelope>, KafkaError>;
}

pub struct StdoutTransport;

pub struct SecureTransport<T: Transport> {
    inner: T,
}

pub struct CommandEnvelope {
    pub id: String,
    pub ts: DateTime<Utc>,
    pub service_id: String,
    pub kind: String,
    pub payload: Value,
    pub signature: Option<String>,  // placeholder
}

pub struct TopicNames {
    pub events: String,   // cc.events.<service_id>
    pub commands: String, // cc.commands.<service_id>
    pub alerts: String,   // cc.alerts
    pub audit: String,    // cc.audit
}

pub enum KafkaError {
    NotAvailable,
    Serialize(serde_json::Error),
    Stub,
}
```

## Что есть

- Transport trait с async send_event / receive_command.
- StdoutTransport: печатает JSON события в stdout (через tracing::info).
- SecureTransport: обёртка, проверяет наличие signature у входящих команд
  (reject если None), но НЕ проверяет саму подпись крипто.
- TopicNames: из service_id → cc.events.\<service_id\>, cc.commands.\<service_id\>,
  cc.alerts, cc.audit.
- CommandEnvelope: десериализация команд от manage.
- Feature-флаг `real` (mock/default): rdkafka + rustls закомментированы как
  заготовка.

## Что TODO

- Ed25519 подпись в SecureTransport:
  - key_id + nonce + timestamp в конверте.
  - replay-protection (nonce + timestamp window).
  - signing key: per-host, запечатанные (TPM/Secure Enclave) — цель.
- Real Kafka transport (feature `real`):
  - rdkafka + rustls, mTLS, ACL на продюсеров.
  - Реализация Transport для KafkaProducer/Consumer.
- Верификация подписи входящих CommandEnvelope (сейчас только проверка
  наличия signature, не крипто).
- Канонический event envelope вместо AgentEvent.
- Оффлайн-буфер (spool_path) при недоступности брокера.
- Per-host FIFO упорядочивание событий.

## Ограничения

- Доверительная граница: подпись здесь — акт утверждения авторства.
  Гости до брокера не достукиваются структурно (mgmt-сегмент без маршрута
  из range).
- StdoutTransport::receive_command() всегда возвращает Ok(None) —
  нет реального poll команд.
- SecureTransport не делает крипто — только обёртка.
- Топики: `cc.events.<service_id>`, не `city.*` (выравнивание с engine — TODO).

## Зависимости

- ccc-core, tokio, tracing, thiserror, serde_json, serde, chrono, async-trait