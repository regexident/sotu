use sotu::prelude::*;
pub struct Counter;
impl Counter {
    fn counting(count: &mut u8) -> Outcome<Phase> {
        *count += 1;
        Handled
    }
}
pub enum Phase {
    Counting { count: u8 },
}
#[automatically_derived]
impl ::core::clone::Clone for Phase {
    #[inline]
    fn clone(&self) -> Phase {
        match self {
            Phase::Counting { count: __self_0 } => {
                Phase::Counting {
                    count: ::core::clone::Clone::clone(__self_0),
                }
            }
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Phase {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Phase::Counting { count: __self_0 } => {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "Counting",
                    "count",
                    &__self_0,
                )
            }
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Phase {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Phase {
    #[inline]
    fn eq(&self, other: &Phase) -> bool {
        match (self, other) {
            (
                Phase::Counting { count: __self_0 },
                Phase::Counting { count: __arg1_0 },
            ) => __self_0 == __arg1_0,
        }
    }
}
impl Phase {
    pub fn counting(count: u8) -> Self {
        Self::Counting { count }
    }
}
impl ::sotu::blocking::IntoStateMachine for Counter {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = Phase;
    type Superstate<'sub> = ();
    type Exit = ::core::convert::Infallible;
    fn initial() -> Phase {
        Phase::counting(0)
    }
}
impl ::sotu::blocking::State<Counter> for Phase {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Counter,
        _event: &(),
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            Phase::Counting { count } => Counter::counting(count),
        }
    }
}
impl ::sotu::private::Submachine for Counter {
    type Event = ();
    type Context<'ctx, 'data: 'ctx> = ();
    type State = Phase;
    type Superstate<'sub> = ();
    type Exit = ::core::convert::Infallible;
    fn initial() -> Phase {
        Phase::counting(0)
    }
}
impl ::sotu::private::BlockingSubmachine for Counter {}
fn main() {
    let mut machine = Counter.state_machine();
    machine.step(&mut ());
}
