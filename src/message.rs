use enum_primitive;
use num::FromPrimitive;

use hex;
use std::fmt;
use std::io;

const MAX_MESSAGE_LENGTH: usize = 32;
const HEADER_SIZE: usize = 7;
const MAX_PAYLOAD: usize = MAX_MESSAGE_LENGTH - HEADER_SIZE;

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    pub enum CommandType {
        PRESENTATION = 0,
        SET = 1,
        REQ = 2,
        INTERNAL = 3,
        STREAM = 4,
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    pub enum CommandSubType {
        StFirmwareConfigRequest = 0,
        StFirmwareConfigResponse = 1,
        StFirmwareRequest = 2,  // Request FW block
        StFirmwareResponse = 3, // Response FW block
    }
}
impl CommandType {
    pub fn _u8(value: u8) -> enum_primitive::Option<CommandType> {
        CommandType::from_u8(value)
    }
}

impl CommandSubType {
    pub fn _u8(value: u8) -> enum_primitive::Option<CommandSubType> {
        CommandSubType::from_u8(value)
    }
}

//"node-id ; child-sensor-id ; command ; ack ; type ; payload \n"
#[derive(Debug)]
pub struct CommandMessage {
    node_id: u8,
    child_sensor_id: u8,
    pub command: u8,
    ack: u8,
    sub_type: u8,
    payload: MessagePayloadType,
}

#[derive(Debug)]
enum MessagePayloadType {
    StreamPayload(MessagePayload),
    OtherPayload(String),
}

impl CommandMessage {
    pub fn new(command_message: &String) -> Result<CommandMessage, String> {
        let message_parts = command_message.split(";").collect::<Vec<&str>>();
        if message_parts.len() < 6 {
            return Err(
                "Invalid Command Message, should have 6 components separated by ';'".to_string(),
            );
        }
        let command_vector = match hex::decode(message_parts[5]) {
            Ok(result) => result,
            _ => return Err("Error while decoding hex".to_string()),
        };
        let array_val = vector_as_u8_32_array(command_vector);
        let command = match message_parts[2].parse::<u8>() {
            Ok(result) => result,
            _ => return Err("Error parsing string to command".to_string()),
        };
        Result::Ok(CommandMessage {
            node_id: match message_parts[0].parse::<u8>() {
                Ok(result) => result,
                _ => return Err("Error parsing string to node_id".to_string()),
            },
            child_sensor_id: match message_parts[1].parse::<u8>() {
                Ok(result) => result,
                _ => return Err("Error parsing string to child_sensor_id".to_string()),
            },
            command: command,
            ack: match message_parts[3].parse::<u8>() {
                Ok(result) => result,
                _ => return Err("Error parsing string to ack".to_string()),
            },
            sub_type: match message_parts[4].parse::<u8>() {
                Ok(result) => result,
                _ => return Err("Error parsing string to sub_type".to_string()),
            },
            payload: match command {
                4 => MessagePayloadType::StreamPayload(MessagePayload {
                    bin_payload: array_val,
                }),
                _ => MessagePayloadType::OtherPayload(String::from(message_parts[5])),
            },
        })
    }
}

union MessagePayload {
    pub message: StreamMessage,
    pub bin_payload: [u8; MAX_MESSAGE_LENGTH],
}

impl MessagePayload {
    pub fn new(bin_payload: [u8; MAX_MESSAGE_LENGTH]) -> MessagePayload {
        MessagePayload {
            bin_payload: bin_payload,
        }
    }
}

struct StreamMessage {
    last: u8,        // 8 bit - Id of last node this message passed
    sender: u8,      // 8 bit - Id of sender node (origin)
    destination: u8, // 8 bit - Id of destination node

    version_length: u8, // 2 bit - Protocol version
    // 1 bit - Signed flag
    // 5 bit - Length of payload
    command_ack_payload: u8, // 3 bit - Command type
    // 1 bit - Request an ack - Indicator that receiver should send an ack back.
    // 1 bit - Is ack messsage - Indicator that this is the actual ack message.
    // 3 bit - Payload data type
    pub _type: u8, // 8 bit - Type varies depending on command
    sensor: u8,    // 8 bit - Id of sensor that this message concerns.

    // Each message can transfer a payload. We add one extra byte for string
    // terminator \0 to be "printable" this is not transferred OTA
    // This union is used to simplify the construction of the binary data types transferred.
    payload: StreamPayload,
}

impl fmt::Debug for MessagePayload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "message: {:?}", unsafe { &self.message })
    }
}

impl fmt::Debug for StreamMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "last: {}, sender: {}, destination: {}, version_length: {}, command_ack_payload: {}, _type: {}, sensor: {}, payload: {:?}", 
        self.last, self.sender, self.destination, self.version_length, self.command_ack_payload, self._type, self.sensor, unsafe{self.payload.data} )
    }
}

union StreamPayload {
    b_value: u8,
    ui_value: u16,
    i_value: i16,
    ul_value: u32,
    l_value: i32,
    msg_float: FloatMessage,
    msg_presentation: PresentationMessage,
    data: [char; MAX_PAYLOAD],
}

struct FloatMessage {
    // Float messages
    f_value: f32,
    f_precision: u8, // Number of decimals when serializing
}

struct PresentationMessage {
    // Presentation messages
    version: u8,     // Library version
    sensor_type: u8, // Sensor type hint for controller, see table above
}

pub fn command_type(message_string: &String) -> Option<CommandType> {
    let message_parts = message_string.split(";").collect::<Vec<&str>>();
    if message_parts.len() == 6 {
        //"node-id ; child-sensor-id ; command ; ack ; type ; payload \n"
        let command_type = message_parts[2].parse::<u8>().unwrap();
        match command_type {
            0 => Some(CommandType::PRESENTATION),
            1 => Some(CommandType::SET),
            2 => Some(CommandType::REQ),
            3 => Some(CommandType::INTERNAL),
            4 => Some(CommandType::STREAM),
            _ => {
                println!("invalid command type {}", command_type);
                None
            }
        }
    } else {
        None
    }
}

pub fn command_sub_type(message_string: &String) -> u8 {
    let message_parts = message_string.split(";").collect::<Vec<&str>>();
    if message_parts.len() == 6 {
        //"node-id ; child-sensor-id ; command ; ack ; type ; payload \n"
        let command_vector = hex::decode(message_parts[5]).unwrap();
        let array_val = vector_as_u8_32_array(command_vector);
        let my_message = MessagePayload::new(array_val);
        println!("{:?}", unsafe { &my_message.bin_payload });
        // println!("{:?}", unsafe{&my_message.message});
        return unsafe { my_message.message._type };
    } else {
        9
    }
}

fn vector_as_u8_32_array(vector: Vec<u8>) -> [u8; 32] {
    let mut arr = [0u8; 32];
    for (place, element) in arr.iter_mut().zip(vector.iter()) {
        *place = *element;
    }
    arr
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn parse_correct_command_type() {
        let message_string = "1;255;4;0;0;FFFFFFFFFFFFFE400102";
        let command_message = CommandMessage::new(&String::from(message_string));
        assert_eq!(
            CommandType::from_u8(command_message.command),
            Some(CommandType::STREAM)
        );
    }

    #[test]
    fn parse_correct_command_sub_type() {
        let message_string = "1;255;4;0;0;FFFFFFFFFFFFFE400102";
        let command_message = CommandMessage::new(&String::from(message_string));
        assert_eq!(
            CommandSubType::from_u8(command_message.sub_type),
            Some(CommandSubType::StFirmwareConfigRequest)
        );
    }
}