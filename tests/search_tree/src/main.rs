mod router;

fn main() {
    let handler = router::handlers::SearchHandler::new();
    handler.run();
}
