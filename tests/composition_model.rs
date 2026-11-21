use std::cell::{Cell, RefCell};

use proptest::prelude::{BoxedStrategy, Just, Strategy, any};
use proptest_state_machine::{ReferenceStateMachine, StateMachineTest, prop_state_machine};
use sotu::prelude::*;
use test_strategy::Arbitrary;

// MARK: Nodes

/// A node of the composed machine, across both levels of composition.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Level {
    Idle,
    WorkMount,
    Group,
    A,
    B,
}

const LEVELS: [Level; 5] = [
    Level::Idle,
    Level::WorkMount,
    Level::Group,
    Level::A,
    Level::B,
];

// The test harness runs each test on its own thread, and every proptest case
// starts by resetting these, so no case observes another's counts or ids.
std::thread_local! {
    static ENTRIES: RefCell<[i64; 5]> = const { RefCell::new([0; 5]) };
    static EXITS: RefCell<[i64; 5]> = const { RefCell::new([0; 5]) };
    static NEXT_ID: Cell<u32> = const { Cell::new(0) };
}

/// Count one entry of `level` on the current thread.
fn record_entry(level: Level) {
    ENTRIES.with_borrow_mut(|entries| entries[level as usize] += 1);
}

/// Count one exit of `level` on the current thread.
fn record_exit(level: Level) {
    EXITS.with_borrow_mut(|exits| exits[level as usize] += 1);
}

/// Return the next instance id of the current thread and advance the counter.
fn allocate_id() -> u32 {
    NEXT_ID.with(|next_id| next_id.replace(next_id.get() + 1))
}

/// Clear all entry and exit counts and continue instance ids at `next_id`.
fn reset_counters(next_id: u32) {
    ENTRIES.with_borrow_mut(|entries| *entries = [0; 5]);
    EXITS.with_borrow_mut(|exits| *exits = [0; 5]);
    NEXT_ID.with(|counter| counter.set(next_id));
}

/// Return how often `level` has been entered minus how often it has been
/// exited.
fn balance(level: Level) -> i64 {
    let entries = ENTRIES.with_borrow(|entries| entries[level as usize]);
    let exits = EXITS.with_borrow(|exits| exits[level as usize]);

    entries - exits
}

/// Return the balance `level` must have while `path` is active: every active
/// node has one unmatched entry, and every other node none.
fn expected_balance(level: Level, path: &[Level]) -> i64 {
    i64::from(path.contains(&level))
}

// MARK: Machines

mod worker {
    use sotu::prelude::*;

    use super::{Level, allocate_id, record_entry, record_exit};

    /// The mounted machine, with leaves `a` and `b` inside the superstate
    /// `group`.
    pub struct Worker {
        pub id: u32,
    }

    impl Default for Worker {
        fn default() -> Self {
            Self { id: allocate_id() }
        }
    }

    pub struct Swap;

    pub struct Reset;

    #[derive(Event)]
    pub enum Event {
        Swap(Swap),
        Reset(Reset),
    }

    #[state_machine(initial = State::a())]
    impl Worker {
        #[state(superstate = group, entry_action = enter_a, exit_action = exit_a)]
        fn a(event: &Event) -> Outcome<State> {
            match event {
                Event::Swap(_) => Transition(State::b()),
                Event::Reset(_) => Super,
            }
        }

        #[state(superstate = group, entry_action = enter_b, exit_action = exit_b)]
        fn b(event: &Event) -> Outcome<State> {
            match event {
                Event::Swap(_) => Transition(State::a()),
                Event::Reset(_) => Super,
            }
        }

        #[superstate(entry_action = enter_group, exit_action = exit_group)]
        fn group(event: &Event) -> Outcome<State> {
            match event {
                Event::Reset(_) => ExternalTransition(State::a()),
                Event::Swap(_) => Super,
            }
        }

        #[action]
        fn enter_a() {
            record_entry(Level::A);
        }

        #[action]
        fn exit_a() {
            record_exit(Level::A);
        }

        #[action]
        fn enter_b() {
            record_entry(Level::B);
        }

        #[action]
        fn exit_b() {
            record_exit(Level::B);
        }

        #[action]
        fn enter_group() {
            record_entry(Level::Group);
        }

        #[action]
        fn exit_group() {
            record_exit(Level::Group);
        }
    }
}

mod plant {
    use sotu::prelude::*;

    use super::{Level, record_entry, record_exit, worker};

    /// The root machine, idling or running a worker.
    #[derive(Default)]
    pub struct Plant {
        pub work: Option<worker::Worker>,
    }

    pub struct Enter;

    pub struct Leave;

    #[derive(Event)]
    pub enum Event {
        Enter(Enter),
        Leave(Leave),
        Swap(worker::Swap),
        Reset(worker::Reset),
    }

    #[state_machine(initial = State::idle())]
    impl Plant {
        #[state(entry_action = enter_idle, exit_action = exit_idle)]
        fn idle(event: &Event) -> Outcome<State> {
            match event {
                Event::Enter(_) => Transition(Superstate::work()),
                _ => Super,
            }
        }

        #[submachine(
            machine = worker::Worker,
            field = work,
            entry_action = enter_work_mount,
            exit_action = exit_work_mount
        )]
        fn work(event: &Event) -> Outcome<State> {
            match event {
                Event::Leave(_) => Transition(State::idle()),
                _ => Super,
            }
        }

        #[action]
        fn enter_idle() {
            record_entry(Level::Idle);
        }

        #[action]
        fn exit_idle() {
            record_exit(Level::Idle);
        }

        #[action]
        fn enter_work_mount() {
            record_entry(Level::WorkMount);
        }

        #[action]
        fn exit_work_mount() {
            record_exit(Level::WorkMount);
        }
    }
}

// MARK: Reference Model

// The model derives every step from the transition rules for mounts alone and
// shares no code with the runtime: a local transition inside an active mount
// keeps its instance, entering a mount creates an instance with the next id,
// and leaving a mount drops it. An external transition from `group` exits and
// enters `group` but not the worker mount, so the worker instance survives it.

#[derive(Clone, Debug)]
struct PlantReference {
    path: Vec<Level>,
    worker_id: Option<u32>,
    next_id: u32,
}

#[derive(Clone, Debug, Arbitrary)]
enum Operation {
    Enter,
    Leave,
    Swap,
    Reset,
}

impl ReferenceStateMachine for PlantReference {
    type State = PlantReference;
    type Transition = Operation;

    fn init_state() -> BoxedStrategy<PlantReference> {
        Just(PlantReference {
            path: vec![Level::Idle],
            worker_id: None,
            next_id: 0,
        })
        .boxed()
    }

    fn transitions(_state: &PlantReference) -> BoxedStrategy<Operation> {
        any::<Operation>().boxed()
    }

    fn apply(mut state: PlantReference, transition: &Operation) -> PlantReference {
        match (transition, state.path.as_slice()) {
            (Operation::Enter, [Level::Idle]) => {
                state.path = worker_path(Level::A);
                state.worker_id = Some(state.allocate_id());
            }
            (Operation::Leave, [Level::WorkMount, ..]) => {
                state.path = vec![Level::Idle];
                state.worker_id = None;
            }
            (Operation::Swap, [.., Level::A]) => {
                state.path = worker_path(Level::B);
            }
            (Operation::Swap, [.., Level::B]) => {
                state.path = worker_path(Level::A);
            }
            // The external transition exits and enters `group`, while the
            // worker mount above stays active.
            (Operation::Reset, [Level::WorkMount, ..]) => {
                state.path = worker_path(Level::A);
            }
            _ => {}
        }

        state
    }
}

impl PlantReference {
    /// Return the id the next created instance receives and advance the
    /// counter.
    fn allocate_id(&mut self) -> u32 {
        let id = self.next_id;

        self.next_id += 1;

        id
    }
}

/// Return the active path ending in the worker leaf `leaf`.
fn worker_path(leaf: Level) -> Vec<Level> {
    vec![Level::WorkMount, Level::Group, leaf]
}

// MARK: System Under Test

/// Return the active path of `machine`, ordered from the root to the leaf.
fn active_path(machine: &InitializedStateMachine<plant::Plant>) -> Vec<Level> {
    let worker_state = match machine.state() {
        plant::State::Idle {} => return vec![Level::Idle],
        plant::State::Work(worker_state) => worker_state,
    };

    let mut path = vec![Level::WorkMount, Level::Group];

    match worker_state {
        worker::State::A {} => path.push(Level::A),
        worker::State::B {} => path.push(Level::B),
    }

    path
}

struct PlantModelTest;

impl StateMachineTest for PlantModelTest {
    type SystemUnderTest = InitializedStateMachine<plant::Plant>;
    type Reference = PlantReference;

    fn init_test(reference: &PlantReference) -> InitializedStateMachine<plant::Plant> {
        reset_counters(reference.next_id);

        plant::Plant::default()
            .uninitialized_state_machine()
            .init(&mut ())
    }

    fn apply(
        mut machine: InitializedStateMachine<plant::Plant>,
        _reference: &PlantReference,
        transition: Operation,
    ) -> InitializedStateMachine<plant::Plant> {
        match transition {
            Operation::Enter => machine.handle(plant::Enter, &mut ()),
            Operation::Leave => machine.handle(plant::Leave, &mut ()),
            Operation::Swap => machine.handle(worker::Swap, &mut ()),
            Operation::Reset => machine.handle(worker::Reset, &mut ()),
        };

        machine
    }

    fn check_invariants(
        machine: &InitializedStateMachine<plant::Plant>,
        reference: &PlantReference,
    ) {
        assert_eq!(active_path(machine), reference.path);
        assert_eq!(
            machine.work.as_ref().map(|worker| worker.id),
            reference.worker_id
        );

        for level in LEVELS {
            assert_eq!(
                balance(level),
                expected_balance(level, &reference.path),
                "balance of {level:?}"
            );
        }
    }
}

// MARK: Tests

prop_state_machine! {
    #[test]
    fn composed_machine_matches_reference_model(sequential 1..=60 => PlantModelTest);
}
