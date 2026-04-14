use swiftlib::task;

fn main() {
    println!("[Terminal] bootstrap ready");
    loop {
        task::yield_now();
    }
}
