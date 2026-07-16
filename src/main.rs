mod collector;

fn main() -> anyhow::Result<()> {
    collector::adapters::outbound::runtime::run()
}
