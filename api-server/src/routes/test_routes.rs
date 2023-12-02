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

pub fn get_test_routes() -> Router {
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

    let router: Router = Router::new().route("/", post({
        let tx = Arc::clone(&tx);
        move |body: String| handler(body, tx)
    }));

    return router;
}
