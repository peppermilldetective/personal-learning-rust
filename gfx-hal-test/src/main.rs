mod engine;

fn main() {
    let win_state = engine::create_default_window();

    engine::run(win_state);
}