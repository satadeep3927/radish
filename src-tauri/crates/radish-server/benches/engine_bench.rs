use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use radish_proto::Frame;
use radish_server::engine::{run, CommandRequest};
use tokio::sync::{mpsc, oneshot};

fn bench_engine_ping(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let (tx, rx) = mpsc::channel(100_000);
    
    rt.spawn(async move {
        run(rx).await;
    });

    c.bench_function("engine PING throughput", |b| {
        b.to_async(&rt).iter_batched(
            || {
                let (resp_tx, resp_rx) = oneshot::channel();
                let req = CommandRequest {
                    frame: Frame::Array(vec![Frame::Bulk(bytes::Bytes::from("PING"))]),
                    responder: resp_tx,
                };
                (req, resp_rx)
            },
            |(req, resp_rx)| {
                let tx_clone = tx.clone();
                async move {
                    let _ = tx_clone.send(req).await;
                    let _ = resp_rx.await;
                }
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_engine_ping);
criterion_main!(benches);
