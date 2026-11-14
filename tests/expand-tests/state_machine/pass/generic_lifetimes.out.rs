use sotu::prelude::*;
pub struct Line<'a>(pub &'a str);
pub enum Input<'a> {
    Line(Line<'a>),
}
impl<'a> ::core::convert::From<Line<'a>> for Input<'a> {
    fn from(value: Line<'a>) -> Self {
        Self::Line(value)
    }
}
impl<'a> ::sotu::event::Variant<Line<'a>> for Input<'a> {
    fn from_variant(value: Line<'a>) -> Self {
        Self::Line(value)
    }
    fn into_variant(self) -> ::core::result::Result<Line<'a>, Self> {
        match self {
            Self::Line(value) => ::core::result::Result::Ok(value),
        }
    }
    fn as_variant_ref(&self) -> ::core::option::Option<&Line<'a>> {
        match self {
            Self::Line(value) => ::core::option::Option::Some(value),
        }
    }
}
impl<'a, SotuParentEvent> ::sotu::event::SubEvent<SotuParentEvent> for Input<'a>
where
    SotuParentEvent: ::sotu::event::Variant<Line<'a>>,
{
    fn accepts(parent: &SotuParentEvent) -> bool {
        <SotuParentEvent as ::sotu::event::Variant<Line<'a>>>::as_variant_ref(parent)
            .is_some()
    }
    fn project(
        parent: SotuParentEvent,
    ) -> ::core::result::Result<Self, SotuParentEvent> {
        let parent = match <SotuParentEvent as ::sotu::event::Variant<
            Line<'a>,
        >>::into_variant(parent) {
            ::core::result::Result::Ok(value) => {
                return ::core::result::Result::Ok(Self::Line(value));
            }
            ::core::result::Result::Err(parent) => parent,
        };
        ::core::result::Result::Err(parent)
    }
    fn embed(self) -> SotuParentEvent {
        match self {
            Self::Line(value) => {
                <SotuParentEvent as ::sotu::event::Variant<
                    Line<'a>,
                >>::from_variant(value)
            }
        }
    }
}
pub struct Reader<'a, 'b> {
    pub prefix: &'b str,
    pub lines: Vec<&'a str>,
}
impl<'a, 'b: 'a> Reader<'a, 'b> {
    fn waiting(event: &Input<'a>) -> Outcome<State<'a>> {
        match event {
            Input::Line(Line(text)) => Transition(State::remembering(*text)),
        }
    }
    fn remembering(event: &Input<'a>) -> Outcome<State<'a>> {
        match event {
            Input::Line(_) => Super,
        }
    }
    fn reading(&mut self, last: &mut &'a str, event: &Input<'a>) -> Outcome<State<'a>> {
        match event {
            Input::Line(Line(text)) => {
                if text.starts_with(self.prefix) {
                    self.lines.push(*last);
                }
                *last = *text;
                Handled
            }
        }
    }
}
pub enum State<'a> {
    Waiting {},
    Remembering { last: &'a str },
}
impl<'a> State<'a> {
    pub fn waiting() -> Self {
        Self::Waiting {}
    }
    pub fn remembering(last: &'a str) -> Self {
        Self::Remembering { last }
    }
}
pub enum Superstate<'sub, 'a> {
    Reading { last: &'sub mut &'a str },
}
impl<'a, 'b: 'a> ::sotu::blocking::IntoStateMachine for Reader<'a, 'b> {
    type Event = Input<'a>;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State<'a>;
    type Superstate<'sub> = Superstate<'sub, 'a> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State<'a> {
        State::waiting()
    }
}
impl<'a, 'b: 'a> ::sotu::blocking::State<Reader<'a, 'b>> for State<'a> {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        _shared_storage: &mut Reader<'a, 'b>,
        event: &Input<'a>,
        _context: &mut (),
    ) -> ::sotu::Outcome<Self, ::core::convert::Infallible> {
        match self {
            State::Waiting {} => Reader::<'a, 'b>::waiting(event),
            State::Remembering { .. } => Reader::<'a, 'b>::remembering(event),
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_, 'a>> {
        match self {
            State::Waiting {} => ::core::option::Option::None,
            State::Remembering { last } => {
                ::core::option::Option::Some(Superstate::Reading { last })
            }
        }
    }
}
impl<'sub, 'a, 'b: 'a> ::sotu::blocking::Superstate<Reader<'a, 'b>>
for Superstate<'sub, 'a> {
    fn call_handler<'ctx, 'data: 'ctx>(
        &mut self,
        shared_storage: &mut Reader<'a, 'b>,
        event: &Input<'a>,
        _context: &mut (),
    ) -> ::sotu::Outcome<State<'a>, ::core::convert::Infallible> {
        match self {
            Superstate::Reading { last } => {
                Reader::<'a, 'b>::reading(shared_storage, last, event)
            }
        }
    }
    fn superstate(&mut self) -> ::core::option::Option<Superstate<'_, 'a>> {
        match self {
            Superstate::Reading { .. } => ::core::option::Option::None,
        }
    }
    fn same_state(&self, other: &Superstate<'_, 'a>) -> bool {
        match (self, other) {
            (Superstate::Reading { .. }, Superstate::Reading { .. }) => true,
        }
    }
}
impl<'a, 'b: 'a> ::sotu::private::Submachine for Reader<'a, 'b> {
    type Event = Input<'a>;
    type Context<'ctx, 'data: 'ctx> = ();
    type State = State<'a>;
    type Superstate<'sub> = Superstate<'sub, 'a> where Self: 'sub;
    type Exit = ::core::convert::Infallible;
    fn initial() -> State<'a> {
        State::waiting()
    }
}
impl<'a, 'b: 'a> ::sotu::private::BlockingSubmachine for Reader<'a, 'b> {}
impl<'a, 'b: 'a> ::sotu::private::ExitlessSubmachine for Reader<'a, 'b> {}
fn main() {
    let prefix = String::from("#");
    let text = String::from("first\n#second");
    let mut machine = Reader {
        prefix: &prefix,
        lines: Vec::new(),
    }
        .state_machine();
    for line in text.lines() {
        machine.handle(Line(line), &mut ());
    }
}
