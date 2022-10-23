use libp2p::core::upgrade::{ProtocolName};

#[derive(Debug, Clone)]
struct FileExchangeProtocol();
#[derive(Clone)]
struct FileExchangeCodec();
#[derive(Debug, Clone, PartialEq, Eq)]
struct FileRequest(String);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResponse(Vec<u8>);

impl ProtocolName for FileExchangeProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/file-exchange/1".as_bytes()
    }
}

