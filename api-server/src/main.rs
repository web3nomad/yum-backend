use chrono;
use rand;
use std::sync::Arc;
use tokio::sync::{
    // watch,
    mpsc,
};
use axum::{
    // routing::get,
    routing::post,
    Router,
    // body,
};

async fn handler(body: String, tx: Arc<mpsc::Sender<String>>) -> &'static str {
    // println!("Send: {}", body);
    // tx.send(body).await.unwrap();
    if let Ok(()) = tx.try_send(body) {
        "Hello World\n"
    } else {
        "Full\n"
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(32);

    let tx = Arc::new(tx);

    tokio::spawn(async move {
        while let Some(body) = rx.recv().await {
            let n = rand::random::<u8>() % 5 + 1;
            tokio::time::sleep(tokio::time::Duration::from_secs(n.into())).await;
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            println!("{} Done! {}", timestamp, body);
        }
    });

    // build our application with a single route
    let app = Router::new().route("/", post({
        let tx = Arc::clone(&tx);
        move |body: String| handler(body, tx)
    }));

    // instance.handler("test".to_string());
    // return "Hello, World!\n";

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

}
