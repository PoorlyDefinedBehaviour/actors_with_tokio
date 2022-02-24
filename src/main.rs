use tokio::sync::{mpsc, oneshot};

struct Actor {
  receiver: mpsc::Receiver<Message>,
  next_id: u32,
}

enum Message {
  GetUniqueId { response_to: oneshot::Sender<u32> },
}

impl Actor {
  pub fn new(receiver: mpsc::Receiver<Message>) -> Self {
    Self {
      receiver,
      next_id: 0,
    }
  }

  fn handle_message(&mut self, message: Message) {
    match message {
      Message::GetUniqueId { response_to } => {
        self.next_id += 1;

        let _ = response_to.send(self.next_id);
      }
    }
  }
}

async fn run_actor(mut actor: Actor) {
  while let Some(message) = actor.receiver.recv().await {
    actor.handle_message(message);
  }
}

#[derive(Clone)]
struct Handle {
  sender: mpsc::Sender<Message>,
}

impl Handle {
  pub fn new() -> Self {
    let (sender, receiver) = mpsc::channel(8);

    let actor = Actor::new(receiver);

    tokio::spawn(run_actor(actor));

    Self { sender }
  }

  pub async fn get_unique_id(&self) -> u32 {
    let (send, recv) = oneshot::channel();

    let message = Message::GetUniqueId { response_to: send };

    let _ = self.sender.send(message).await;
    recv.await.expect("Actor has been killed")
  }
}

#[tokio::main]
async fn main() {
  let handle = Handle::new();

  dbg!(handle.get_unique_id().await); // 1
  dbg!(handle.get_unique_id().await); // 2
  dbg!(handle.get_unique_id().await); // 3
}
