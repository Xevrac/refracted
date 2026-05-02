use crate::blaze::tdf::TdfEncoder;
use crate::common::error::BlazeResult;
use bytes::Bytes;

pub fn handle_messaging_send_message(_payload: &[u8]) -> BlazeResult<Bytes> {
    use crate::session::get_next_message_id;

    let mut response = Vec::new();
    let mgid = get_next_message_id();
    response.extend_from_slice(&TdfEncoder::encode_int("MGID", mgid as i32));
    let mids_list = vec![mgid as i32];
    response.extend_from_slice(&TdfEncoder::encode_list("MIDS", &mids_list));
    Ok(Bytes::from(response))
}
