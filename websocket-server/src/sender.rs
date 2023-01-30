use std::io::Result as IoResult;
use std::io::Write;
use std::net::Shutdown;
use websocket_core::action::sender::Sender as SenderAble;
use websocket_core::protocol::dataframe::DataFrame;
use websocket_core::protocol::message::Message;
use websocket_core::stream::AsTcpStream;
use crate::WebSocketResult;

pub struct Writer<W> {
	pub stream: W,

	pub sender: Sender,
}

pub struct Sender {
	mask: bool,
}

impl Sender {
	pub fn new(mask: bool) -> Sender {
		Sender { mask }
	}
}

impl SenderAble for Sender {
	fn is_masked(&self) -> bool {
		self.mask
	}
}

impl<W> Writer<W>
where
	W: Write,
{
	pub fn send_dataframe<D>(&mut self, dataframe: &D) -> WebSocketResult<()>
	where
		D: DataFrame,
		W: Write,
	{
		self.sender.send_dataframe(&mut self.stream, dataframe)
	}

	/// Sends a single message to the remote endpoint.
	pub fn send_message<M>(&mut self, message: &M) -> WebSocketResult<()>
	where
		M: Message,
	{
		self.sender.send_message(&mut self.stream, message)
	}
}

impl<S> Writer<S>
where
	S: AsTcpStream + Write,
{

	pub fn shutdown(&self) -> IoResult<()> {
		self.stream.as_tcp().shutdown(Shutdown::Write)
	}

	pub fn shutdown_all(&self) -> IoResult<()> {
		self.stream.as_tcp().shutdown(Shutdown::Both)
	}
}

