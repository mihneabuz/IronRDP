use std::io;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use thiserror::Error;

use crate::PduParsing;

pub mod fast_path;
pub mod mouse;
pub mod mouse_rel;
pub mod mouse_x;
pub mod scan_code;
pub mod sync;
pub mod unicode;
pub mod unused;

pub use self::mouse::MousePdu;
pub use self::mouse_rel::MouseRelPdu;
pub use self::mouse_x::MouseXPdu;
pub use self::scan_code::ScanCodePdu;
pub use self::sync::SyncPdu;
pub use self::unicode::UnicodePdu;
pub use self::unused::UnusedPdu;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputEventPdu(pub Vec<InputEvent>);

impl PduParsing for InputEventPdu {
    type Error = InputEventError;

    fn from_buffer(mut stream: impl io::Read) -> Result<Self, Self::Error> {
        let number_of_events = stream.read_u16::<LittleEndian>()?;
        let _padding = stream.read_u16::<LittleEndian>()?;

        let events = (0..number_of_events)
            .map(|_| InputEvent::from_buffer(&mut stream))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self(events))
    }

    fn to_buffer(&self, mut stream: impl io::Write) -> Result<(), Self::Error> {
        stream.write_u16::<LittleEndian>(self.0.len() as u16)?;
        stream.write_u16::<LittleEndian>(0)?; // padding

        for event in self.0.iter() {
            event.to_buffer(&mut stream)?;
        }

        Ok(())
    }

    fn buffer_length(&self) -> usize {
        4 + self.0.iter().map(PduParsing::buffer_length).sum::<usize>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    Sync(SyncPdu),
    Unused(UnusedPdu),
    ScanCode(ScanCodePdu),
    Unicode(UnicodePdu),
    Mouse(MousePdu),
    MouseX(MouseXPdu),
    MouseRel(MouseRelPdu),
}

impl PduParsing for InputEvent {
    type Error = InputEventError;

    fn from_buffer(mut stream: impl io::Read) -> Result<Self, Self::Error> {
        let _event_time = stream.read_u32::<LittleEndian>()?; // ignored by a server
        let event_type = stream.read_u16::<LittleEndian>()?;
        let event_type =
            InputEventType::from_u16(event_type).ok_or(InputEventError::InvalidInputEventType(event_type))?;

        match event_type {
            InputEventType::Sync => Ok(Self::Sync(SyncPdu::from_buffer(&mut stream)?)),
            InputEventType::Unused => Ok(Self::Unused(UnusedPdu::from_buffer(&mut stream)?)),
            InputEventType::ScanCode => Ok(Self::ScanCode(ScanCodePdu::from_buffer(&mut stream)?)),
            InputEventType::Unicode => Ok(Self::Unicode(UnicodePdu::from_buffer(&mut stream)?)),
            InputEventType::Mouse => Ok(Self::Mouse(MousePdu::from_buffer(&mut stream)?)),
            InputEventType::MouseX => Ok(Self::MouseX(MouseXPdu::from_buffer(&mut stream)?)),
            InputEventType::MouseRel => Ok(Self::MouseRel(MouseRelPdu::from_buffer(&mut stream)?)),
        }
    }

    fn to_buffer(&self, mut stream: impl io::Write) -> Result<(), Self::Error> {
        stream.write_u32::<LittleEndian>(0)?; // event time is ignored by a server
        stream.write_u16::<LittleEndian>(InputEventType::from(self).to_u16().unwrap())?;

        match self {
            Self::Sync(pdu) => pdu.to_buffer(&mut stream),
            Self::Unused(pdu) => pdu.to_buffer(&mut stream),
            Self::ScanCode(pdu) => pdu.to_buffer(&mut stream),
            Self::Unicode(pdu) => pdu.to_buffer(&mut stream),
            Self::Mouse(pdu) => pdu.to_buffer(&mut stream),
            Self::MouseX(pdu) => pdu.to_buffer(&mut stream),
            Self::MouseRel(pdu) => pdu.to_buffer(&mut stream),
        }
    }

    fn buffer_length(&self) -> usize {
        6 + match self {
            Self::Sync(pdu) => pdu.buffer_length(),
            Self::Unused(pdu) => pdu.buffer_length(),
            Self::ScanCode(pdu) => pdu.buffer_length(),
            Self::Unicode(pdu) => pdu.buffer_length(),
            Self::Mouse(pdu) => pdu.buffer_length(),
            Self::MouseX(pdu) => pdu.buffer_length(),
            Self::MouseRel(pdu) => pdu.buffer_length(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive, ToPrimitive)]
#[repr(u16)]
enum InputEventType {
    Sync = 0x0000,
    Unused = 0x0002,
    ScanCode = 0x0004,
    Unicode = 0x0005,
    Mouse = 0x8001,
    MouseX = 0x8002,
    MouseRel = 0x8004,
}

impl From<&InputEvent> for InputEventType {
    fn from(event: &InputEvent) -> Self {
        match event {
            InputEvent::Sync(_) => Self::Sync,
            InputEvent::Unused(_) => Self::Unused,
            InputEvent::ScanCode(_) => Self::ScanCode,
            InputEvent::Unicode(_) => Self::Unicode,
            InputEvent::Mouse(_) => Self::Mouse,
            InputEvent::MouseX(_) => Self::MouseX,
            InputEvent::MouseRel(_) => Self::MouseRel,
        }
    }
}

#[derive(Debug, Error)]
pub enum InputEventError {
    #[error("IO error")]
    IOError(#[from] io::Error),
    #[error("invalid Input Event type: {0}")]
    InvalidInputEventType(u16),
    #[error("encryption not supported")]
    EncryptionNotSupported,
    #[error("event code not supported {0}")]
    EventCodeUnsupported(u8),
    #[error("keyboard flags not supported {0}")]
    KeyboardFlagsUnsupported(u8),
    #[error("synchronize flags not supported {0}")]
    SynchronizeFlagsUnsupported(u8),
    #[error("Fast-Path Input Event PDU is empty")]
    EmptyFastPathInput,
}
