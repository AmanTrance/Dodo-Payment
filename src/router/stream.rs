use hyper::body::{Body, Bytes, Frame};
use std::task::Poll;

pub(crate) struct EventStreamBody {
    receiver: tokio::sync::mpsc::Receiver<Bytes>,
    is_finished: bool,
}

impl EventStreamBody {
    pub(crate) fn new(receiver: tokio::sync::mpsc::Receiver<Bytes>) -> Self {
        Self {
            receiver,
            is_finished: false,
        }
    }
}

impl Body for EventStreamBody {
    type Data = Bytes;
    type Error = hyper::http::Error;

    fn poll_frame(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        match self.as_mut().get_mut().receiver.poll_recv(cx) {
            Poll::Ready(None) => {
                self.as_mut().get_mut().is_finished = true;
                Poll::Ready(None)
            }

            Poll::Ready(Some(data)) => Poll::Ready(Some(Ok(Frame::data(data)))),

            Poll::Pending => Poll::Pending,
        }
    }

    fn is_end_stream(&self) -> bool {
        self.is_finished
    }
}
