use dlt_core::dlt::{
    self, Argument, DltTimeStamp, Endianness, LogLevel, Message, PayloadContent, StandardHeader,
    StorageHeader, StringCoding, TypeInfo, TypeInfoKind, Value,
};
use futures::SinkExt;
use tokio_dlt::{DltTcpClient, DltTcpClientOptions, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts = DltTcpClientOptions {
        host: "127.0.0.1".to_string(),
        port: 3490,
    };
    let dlt_tcp_client = DltTcpClient::connect(&opts).await?;
    let mut sink = dlt_tcp_client.write();

    let timestamp = DltTimeStamp {
        seconds: 0x4DC9_2C26,
        microseconds: 0x000C_A2D8,
    };
    let storage_header = StorageHeader {
        timestamp,
        ecu_id: "abc".to_string(),
    };

    let extended_header = dlt::ExtendedHeader {
        argument_count: 2,
        verbose: true,
        message_type: dlt::MessageType::Log(LogLevel::Warn),
        application_id: "abc".to_string(),
        context_id: "CON".to_string(),
    };
    let type_info = TypeInfo {
        kind: TypeInfoKind::Bool,
        coding: StringCoding::UTF8,
        has_variable_info: true,
        has_trace_info: false,
    };
    let argument = Argument {
        type_info,
        name: Some("foo".to_string()),
        unit: None,
        fixed_point: None,
        value: Value::Bool(1),
    };
    let payload = PayloadContent::Verbose(vec![argument]);

    let header: StandardHeader = StandardHeader {
        version: 1,
        endianness: Endianness::Big,
        has_extended_header: false,
        payload_length: 0x1,
        message_counter: 0x33,
        ecu_id: Some("abc".to_string()),
        session_id: None,
        timestamp: Some(5),
    };

    let message = Message {
        storage_header: Some(storage_header),
        header,
        extended_header: Some(extended_header),
        payload,
    };

    sink.send(message).await?;

    Ok(())
}
