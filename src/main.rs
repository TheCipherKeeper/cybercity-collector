mod collector;

fn main() {
    collector::adapters::outbound::runtime::run();
}
