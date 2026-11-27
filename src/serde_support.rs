//! `serde` support for the blocking and, with the `async` feature, the
//! awaitable state machine wrappers.
//!
//! Every wrapper serializes as a struct named `StateMachine` with the fields
//! `shared_storage`, `state` and `exit`, in that order. Storage of mounted
//! machines is part of `shared_storage`, because it lives in fields of the
//! parent's storage. Handlers, hooks and the lazy initialization flag are not
//! serialized. Neither is the progress that an awaitable machine records for an
//! interrupted initialization, transition or termination.
//!
//! [`UninitializedStateMachine`] and [`StateMachine`] implement `Deserialize`,
//! and both come back uninitialized, because the entry actions that built the
//! serialized state are not part of the data. Initializing such a machine runs
//! the entry actions of every level of the deserialized state, outermost first,
//! and creates a mounted instance only when its field is `None`. When the
//! deserialized `exit` is `Some`, initialization runs no action and leaves a
//! terminated machine. [`InitializedStateMachine`] implements only `Serialize`,
//! since deserializing into it would skip the entry actions.
//!
//! The deserializer requires all three fields, and so rejects data in which
//! `exit` is missing instead of reading it as a running machine. It ignores
//! unknown fields.

use core::fmt;
use core::marker::PhantomData;

use serde::de::{self, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "async")]
use crate::awaitable;
use crate::blocking::{
    InitializedStateMachine, IntoStateMachine, StateMachine, UninitializedStateMachine,
};

const NAME: &str = "StateMachine";

const FIELDS: &[&str] = &["shared_storage", "state", "exit"];

/// The shared storage, the leaf state and the exit value of a machine.
///
/// The blocking and the awaitable machines name the same associated types
/// through different traits, so the helpers take them as plain parameters.
type Parts<M, S, X> = (M, S, Option<X>);

// MARK: Serialization

fn serialize_parts<M, S, X, Z>(
    serializer: Z,
    (shared_storage, state, exit): (&M, &S, Option<&X>),
) -> Result<Z::Ok, Z::Error>
where
    M: Serialize,
    S: Serialize,
    X: Serialize,
    Z: Serializer,
{
    let mut record = serializer.serialize_struct(NAME, FIELDS.len())?;

    record.serialize_field(FIELDS[0], shared_storage)?;
    record.serialize_field(FIELDS[1], state)?;
    record.serialize_field(FIELDS[2], &exit)?;

    record.end()
}

impl<M> Serialize for UninitializedStateMachine<M>
where
    M: IntoStateMachine + Serialize,
    M::State: Serialize,
    M::Exit: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_parts(serializer, self.parts())
    }
}

impl<M> Serialize for InitializedStateMachine<M>
where
    M: IntoStateMachine + Serialize,
    M::State: Serialize,
    M::Exit: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_parts(serializer, self.parts())
    }
}

impl<M> Serialize for StateMachine<M>
where
    M: IntoStateMachine + Serialize,
    M::State: Serialize,
    M::Exit: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_parts(serializer, self.parts())
    }
}

#[cfg(feature = "async")]
impl<M> Serialize for awaitable::UninitializedStateMachine<M>
where
    M: awaitable::IntoStateMachine + Serialize,
    M::State: Serialize,
    M::Exit: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_parts(serializer, self.parts())
    }
}

#[cfg(feature = "async")]
impl<M> Serialize for awaitable::InitializedStateMachine<M>
where
    M: awaitable::IntoStateMachine + Serialize,
    M::State: Serialize,
    M::Exit: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_parts(serializer, self.parts())
    }
}

#[cfg(feature = "async")]
impl<M> Serialize for awaitable::StateMachine<M>
where
    M: awaitable::IntoStateMachine + Serialize,
    M::State: Serialize,
    M::Exit: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_parts(serializer, self.parts())
    }
}

// MARK: Deserialization

/// A key of the serialized struct.
enum Field {
    SharedStorage,
    State,
    Exit,
    Unknown,
}

struct FieldVisitor;

impl Visitor<'_> for FieldVisitor {
    type Value = Field;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("`shared_storage`, `state` or `exit`")
    }

    fn visit_u64<E: de::Error>(self, index: u64) -> Result<Field, E> {
        Ok(match index {
            0 => Field::SharedStorage,
            1 => Field::State,
            2 => Field::Exit,
            _ => Field::Unknown,
        })
    }

    fn visit_str<E: de::Error>(self, name: &str) -> Result<Field, E> {
        Ok(match name {
            "shared_storage" => Field::SharedStorage,
            "state" => Field::State,
            "exit" => Field::Exit,
            _ => Field::Unknown,
        })
    }

    fn visit_bytes<E: de::Error>(self, name: &[u8]) -> Result<Field, E> {
        Ok(match name {
            b"shared_storage" => Field::SharedStorage,
            b"state" => Field::State,
            b"exit" => Field::Exit,
            _ => Field::Unknown,
        })
    }
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_identifier(FieldVisitor)
    }
}

/// Marks a type that produces [`Parts`] without owning any of them.
type Produces<M, S, X> = PhantomData<fn() -> Parts<M, S, X>>;

/// Reads the three fields from a map or a sequence.
struct PartsVisitor<M, S, X>(Produces<M, S, X>);

impl<'de, M, S, X> Visitor<'de> for PartsVisitor<M, S, X>
where
    M: Deserialize<'de>,
    S: Deserialize<'de>,
    X: Deserialize<'de>,
{
    type Value = Parts<M, S, X>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a state machine with the fields `shared_storage`, `state` and `exit`")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Parts<M, S, X>, A::Error> {
        let shared_storage = sequence
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let state = sequence
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let exit = sequence
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;

        Ok((shared_storage, state, exit))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Parts<M, S, X>, A::Error> {
        let mut shared_storage = None;
        let mut state = None;
        let mut exit: Option<Option<X>> = None;

        while let Some(field) = map.next_key()? {
            match field {
                Field::SharedStorage => {
                    if shared_storage.is_some() {
                        return Err(de::Error::duplicate_field(FIELDS[0]));
                    }

                    shared_storage = Some(map.next_value()?);
                }
                Field::State => {
                    if state.is_some() {
                        return Err(de::Error::duplicate_field(FIELDS[1]));
                    }

                    state = Some(map.next_value()?);
                }
                Field::Exit => {
                    if exit.is_some() {
                        return Err(de::Error::duplicate_field(FIELDS[2]));
                    }

                    exit = Some(map.next_value()?);
                }
                Field::Unknown => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }

        let shared_storage = shared_storage.ok_or_else(|| de::Error::missing_field(FIELDS[0]))?;
        let state = state.ok_or_else(|| de::Error::missing_field(FIELDS[1]))?;
        let exit = exit.ok_or_else(|| de::Error::missing_field(FIELDS[2]))?;

        Ok((shared_storage, state, exit))
    }
}

fn deserialize_parts<'de, M, S, X, D>(deserializer: D) -> Result<Parts<M, S, X>, D::Error>
where
    M: Deserialize<'de>,
    S: Deserialize<'de>,
    X: Deserialize<'de>,
    D: Deserializer<'de>,
{
    deserializer.deserialize_struct(NAME, FIELDS, PartsVisitor(PhantomData))
}

impl<'de, M> Deserialize<'de> for UninitializedStateMachine<M>
where
    M: IntoStateMachine + Deserialize<'de>,
    M::State: Deserialize<'de>,
    M::Exit: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (shared_storage, state, exit) = deserialize_parts(deserializer)?;

        Ok(Self::from_parts(shared_storage, state, exit))
    }
}

impl<'de, M> Deserialize<'de> for StateMachine<M>
where
    M: IntoStateMachine + Deserialize<'de>,
    M::State: Deserialize<'de>,
    M::Exit: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (shared_storage, state, exit) = deserialize_parts(deserializer)?;

        Ok(Self::from_parts(shared_storage, state, exit))
    }
}
