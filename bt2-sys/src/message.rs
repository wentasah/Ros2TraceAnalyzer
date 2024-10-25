use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use derive_more::derive::{Deref, From};

use crate::clock_snapshot::{BtClockClassConst, BtClockSnapshotConst};
use crate::event::BtEventConst;
use crate::raw_bindings::{
    bt_message, bt_message_event_borrow_default_clock_snapshot_const,
    bt_message_event_borrow_event_const,
    bt_message_event_borrow_stream_class_default_clock_class_const, bt_message_get_ref,
    bt_message_get_type, bt_message_put_ref, bt_message_type,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtMessageType {
    StreamBeginning,
    StreamEnd,
    Event,
    PacketBeginning,
    PacketEnd,
    DiscardedEvents,
    DiscardedPackets,
    MessageIteratorInactivity,
}

impl From<bt_message_type> for BtMessageType {
    fn from(value: bt_message_type) -> Self {
        match value {
            bt_message_type::BT_MESSAGE_TYPE_STREAM_BEGINNING => BtMessageType::StreamBeginning,
            bt_message_type::BT_MESSAGE_TYPE_EVENT => BtMessageType::Event,
            bt_message_type::BT_MESSAGE_TYPE_STREAM_END => BtMessageType::StreamEnd,
            bt_message_type::BT_MESSAGE_TYPE_PACKET_BEGINNING => BtMessageType::PacketBeginning,
            bt_message_type::BT_MESSAGE_TYPE_PACKET_END => BtMessageType::PacketEnd,
            bt_message_type::BT_MESSAGE_TYPE_DISCARDED_EVENTS => BtMessageType::DiscardedEvents,
            bt_message_type::BT_MESSAGE_TYPE_DISCARDED_PACKETS => BtMessageType::DiscardedPackets,
            bt_message_type::BT_MESSAGE_TYPE_MESSAGE_ITERATOR_INACTIVITY => {
                BtMessageType::MessageIteratorInactivity
            }
            _ => {
                // All bt_message_type variants are handled above
                unreachable!("Bug: unknown bt_message_type = {}", value.0);
            }
        }
    }
}

#[repr(transparent)]
pub struct BtMessageConst(ConstNonNull<bt_message>);

#[derive(From, Clone)]
pub enum BtMessageConstCast {
    StreamBeginning(BtStreamBeginningMessageConst),
    StreamEnd(BtStreamEndMessageConst),
    Event(BtEventMessageConst),
    PacketBeginning(BtPacketBeginningMessageConst),
    PacketEnd(BtPacketEndMessageConst),
    DiscardedEvents(BtDiscardedEventsMessageConst),
    DiscardedPackets(BtDiscardedPacketsMessageConst),
    MessageIteratorInactivity(BtMessageIteratorInactivityMessageConst),
}

#[repr(transparent)]
#[derive(Deref, Clone)]
pub struct BtStreamBeginningMessageConst(BtMessageConst);

#[repr(transparent)]
#[derive(Deref, Clone)]
pub struct BtStreamEndMessageConst(BtMessageConst);

#[repr(transparent)]
#[derive(Deref, Clone)]
pub struct BtEventMessageConst(BtMessageConst);

#[repr(transparent)]
#[derive(Deref, Clone)]
pub struct BtPacketBeginningMessageConst(BtMessageConst);

#[repr(transparent)]
#[derive(Deref, Clone)]
pub struct BtPacketEndMessageConst(BtMessageConst);

#[repr(transparent)]
#[derive(Deref, Clone)]
pub struct BtDiscardedEventsMessageConst(BtMessageConst);

#[repr(transparent)]
#[derive(Deref, Clone)]
pub struct BtDiscardedPacketsMessageConst(BtMessageConst);

#[repr(transparent)]
#[derive(Deref, Clone)]
pub struct BtMessageIteratorInactivityMessageConst(BtMessageConst);

impl BtMessageConst {
    pub(crate) unsafe fn new_unchecked(message: *const bt_message) -> BtMessageConst {
        Self(ConstNonNull::new_unchecked(message))
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const bt_message {
        self.0.as_ptr()
    }

    #[must_use]
    pub fn get_type(&self) -> BtMessageType {
        unsafe { bt_message_get_type(self.as_ptr()) }.into()
    }

    #[must_use]
    pub fn cast(self) -> BtMessageConstCast {
        match self.get_type() {
            BtMessageType::StreamBeginning => BtStreamBeginningMessageConst(self).into(),
            BtMessageType::StreamEnd => BtStreamEndMessageConst(self).into(),
            BtMessageType::Event => BtEventMessageConst(self).into(),
            BtMessageType::PacketBeginning => BtPacketBeginningMessageConst(self).into(),
            BtMessageType::PacketEnd => BtPacketEndMessageConst(self).into(),
            BtMessageType::DiscardedEvents => BtDiscardedEventsMessageConst(self).into(),
            BtMessageType::DiscardedPackets => BtDiscardedPacketsMessageConst(self).into(),
            BtMessageType::MessageIteratorInactivity => {
                BtMessageIteratorInactivityMessageConst(self).into()
            }
        }
    }

    /// Cast this message to a [`BtEventMessageConst`].
    ///
    /// # Panics
    /// Panics if the message is not of type [`BtMessageType::Event`].
    #[must_use]
    pub fn into_event_msg(self) -> BtEventMessageConst {
        match self.cast() {
            BtMessageConstCast::Event(event) => event,
            _ => panic!("Message is not of type Event"),
        }
    }
}

impl BtEventMessageConst {
    /// Get the event contained in this message.
    #[must_use]
    pub fn get_event<'a>(&'a self) -> BtEventConst {
        debug_assert!(!self.as_ptr().is_null());
        unsafe {
            BtEventConst::<'a>::new_unchecked(bt_message_event_borrow_event_const(self.as_ptr()))
        }
    }

    /// Get snapshot of the default clock.
    ///
    /// # Panics
    /// If the stream class of the event does not have a default clock class,
    /// i.e., if [`Self::get_default_clock_class`] returns `None`.
    #[must_use]
    pub fn get_default_clock_snapshot(&self) -> BtClockSnapshotConst {
        assert!(self.get_default_clock_class().is_some());
        debug_assert!(!self.as_ptr().is_null());
        unsafe {
            BtClockSnapshotConst::new_unchecked(
                bt_message_event_borrow_default_clock_snapshot_const(self.as_ptr()),
            )
        }
    }

    /// Get the default clock class of the stream class of the event.
    #[must_use]
    pub fn get_default_clock_class(&self) -> Option<BtClockClassConst> {
        unsafe {
            bt_message_event_borrow_stream_class_default_clock_class_const(self.as_ptr())
                .try_into()
                .ok()
                .map(|ptr| BtClockClassConst::new_unchecked(ptr))
        }
    }
}

impl Clone for BtMessageConst {
    fn clone(&self) -> Self {
        debug_assert!(!self.0.is_null());
        unsafe {
            bt_message_get_ref(self.0);
        }

        unsafe { Self::new_unchecked(self.0) }
    }
}

impl Drop for BtMessageConst {
    fn drop(&mut self) {
        unsafe {
            bt_message_put_ref(self.0);
        }
    }
}

pub(crate) struct BtMessageArrayConst(NonNull<*const bt_message>, usize);
impl BtMessageArrayConst {
    pub(crate) unsafe fn new_unchecked(
        messages: *mut *const bt_message,
        count: u64,
    ) -> BtMessageArrayConst {
        BtMessageArrayConst(NonNull::new(messages).unwrap(), count.try_into().unwrap())
    }
}

impl Deref for BtMessageArrayConst {
    type Target = [BtMessageConst];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.0.cast().as_ptr(), self.1) }
    }
}

impl DerefMut for BtMessageArrayConst {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.0.cast().as_ptr(), self.1) }
    }
}

impl Default for BtMessageArrayConst {
    fn default() -> Self {
        Self(NonNull::dangling(), 0)
    }
}

impl Drop for BtMessageArrayConst {
    fn drop(&mut self) {
        for message in &mut **self {
            unsafe {
                std::ptr::drop_in_place(message);
            }
        }
    }
}

impl Debug for BtMessageArrayConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter().map(|m| m.0)).finish()
    }
}
