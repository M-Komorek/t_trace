use t_trace::state::DaemonState; // <-- RENAMED

fn main() {
    println!("Hello, t_trace!");
    let _state = DaemonState::default(); // Now using the new name
}

