//! `serde` support for the blocking state machine wrappers.
//!
//! Every wrapper serializes as a struct named `StateMachine` with the fields
//! `shared_storage`, `state` and `exit`, in that order. Storage of mounted
//! machines is part of `shared_storage`, because it lives in fields of the
//! parent's storage. Handlers, hooks and the lazy initialization flag are not
//! serialized.
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

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::blocking::{
    InitializedStateMachine, IntoStateMachine, StateMachine, UninitializedStateMachine,
};

const NAME: &str = "StateMachine";

const FIELDS: &[&str] = &["shared_storage", "state", "exit"];

// MARK: Serialization

fn serialize_parts<M, S>(
    serializer: S,
    (shared_storage, state, exit): (&M, &M::State, Option<&M::Exit>),
) -> Result<S::Ok, S::Error>
where
    M: IntoStateMachine + Serialize,
    M::State: Serialize,
    M::Exit: Serialize,
    S: Serializer,
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
