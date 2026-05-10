use crate::blaze::components;
use crate::blaze::protocol::Fire2FramePacket;
use crate::common::discovery;
use crate::common::error::{BlazeError, BlazeResult};
use bytes::Bytes;


/// Handle unhandled command - returns empty response and logs
fn handle_unhandled_command(
    component: u16,
    command: u16,
    payload: &[u8],
    incoming_seq: Option<u64>,
) -> BlazeResult<Bytes> {
    let component_name = components::get_component_name(component);
    let command_name = components::get_command_name(component, command)
        .unwrap_or_else(|| format!("UnknownCommand({})", command));
    
    let is_new = discovery::check_and_record(component, command);
    discovery::record_first_seen_seq_if_new(is_new, component, command, incoming_seq);
    
    if is_new {
        crate::console_println!(
            "\x1b[38;2;255;215;0m[\u{1F50D} DISCOVERY]\x1b[0m NEW Component={} ({}), Command={} ({}), PayloadSize={}",
            component,
            component_name,
            command,
            command_name,
            payload.len()
        );
    } else {
        crate::console_println!(
            "\x1b[38;2;255;165;0m[UNHANDLED]\x1b[0m {} Component={} ({}), Command={}, PayloadSize={}",
            command_name,
            component,
            component_name,
            command,
            payload.len()
        );
    }
    
    // Return empty response for unhandled commands
    Ok(Bytes::from(Vec::new()))
}

/// Route packet to appropriate handler
pub fn handle_packet(packet: &Fire2FramePacket, incoming_seq: Option<u64>) -> BlazeResult<Bytes> {
    handle_packet_fields(
        packet.header.component,
        packet.header.command,
        &packet.payload,
        incoming_seq,
    )
}

pub fn handle_packet_fields(
    component: u16,
    command: u16,
    payload: &[u8],
    incoming_seq: Option<u64>,
) -> BlazeResult<Bytes> {
    // Client-first dispatch: each title module can fully shape any command/response.
    if let Some(result) = crate::client::handle_packet_fields(component, command, payload) {
        return result;
    }

    match (component, command) {
        (0, 0) => Err(BlazeError::ConnectionClosed),
        _ => handle_unhandled_command(component, command, payload, incoming_seq),
    }
}



