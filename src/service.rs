
#[derive(Clone)]
pub struct DBWorker {
    pub receiver: crossbeam_channel::Receiver<usize>,
}

impl DBWorker {
    pub fn new(url: &str, recv: crossbeam_channel::Receiver<usize>) -> Self {
        Self { receiver: recv, }
    }
    pub fn handle_message(&mut self, message: usize) {
        println!("Received message: {}", message);
    }
}

pub fn run_service(mut actor: DBWorker) {
    while let Ok(message) = actor.receiver.recv() {
        actor.handle_message(message);
    }
}
