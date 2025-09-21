use cosmic::{
    iced::{Subscription, futures::SinkExt},
    iced_futures::MaybeSend,
};
use pop_launcher_service::{Args, IpcClient};
use std::hash::Hash;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::{Stream, StreamExt};

#[derive(Debug, Clone)]
pub enum Request {
    Search(String),
    ServiceIsClosed,
}

#[derive(Debug, Clone)]
pub enum Event {
    Started(mpsc::Sender<Request>),
    Response(pop_launcher::Response),
    ServiceIsClosed,
}

pub fn subscription<I: 'static + Hash + Copy + Send + Sync>(
    id: I,
) -> cosmic::iced::Subscription<Event> {
    Subscription::run_with_id(
        id,
        cosmic::iced_futures::stream::channel(1, |mut output| async move {
            loop {
                tracing::info!("starting pop-launcher service");
                let mut responses = std::pin::pin!(service());
                while let Some(message) = responses.next().await {
                    let _res = output.send(message).await;
                }
            }
        }),
    )
}

pub fn service() -> impl Stream<Item = Event> + MaybeSend {
    let (requests_tx, mut requests_rx) = mpsc::channel(4);
    let (responses_tx, responses_rx) = mpsc::channel(4);

    let service_future = async move {
        let _res = responses_tx.send(Event::Started(requests_tx.clone())).await;

        let client = &mut None;

        while let Some(request) = requests_rx.recv().await {
            match request {
                Request::Search(s) => {
                    tracing::info!("Searched {s}");
                    if let Some((client, _)) = client_request(&responses_tx, client) {
                        let _res = client.send(pop_launcher::Request::Search(s)).await;
                    }
                }
                Request::ServiceIsClosed => {
                    *client = None;
                }
            }
        }
    };

    let _res = tokio::task::spawn(service_future);

    tokio_stream::wrappers::ReceiverStream::new(responses_rx)
}

/// Initializes pop-launcher if it is not running, and returns a handle to its client.
fn client_request<'a>(
    tx: &mpsc::Sender<Event>,
    client: &'a mut Option<(IpcClient, oneshot::Sender<()>)>,
) -> &'a mut Option<(IpcClient, oneshot::Sender<()>)> {
    if client.is_none() {
        *client = match pop_launcher_service::IpcClient::new_with_args(Args {
            max_files: 20,
            max_open: 99,
            max_search: 20,
        }) {
            Ok((new_client, responses)) => {
                let tx = tx.clone();

                let (kill_tx, kill_rx) = tokio::sync::oneshot::channel();
                let listener = async {
                    tracing::info!("starting pop-launcher instance");
                    let listener = Box::pin(async move {
                        let mut responses = std::pin::pin!(responses);
                        while let Some(response) = responses.next().await {
                            _ = tx.send(Event::Response(response)).await;
                        }
                        _ = tx.send(Event::ServiceIsClosed).await;
                    });

                    let killswitch = Box::pin(async move {
                        let _res = kill_rx.await;
                    });

                    futures::future::select(listener, killswitch).await;
                };

                let _res = tokio::task::spawn(listener);

                Some((new_client, kill_tx))
            }
            Err(why) => {
                tracing::error!("pop-launcher failed to start: {}", why);
                None
            }
        }
    };

    client
}
