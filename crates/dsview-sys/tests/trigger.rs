//! Trigger boundary tests against the source-built runtime library.
//!
//! The DSView trigger setters only touch the process-global `struct ds_trigger`,
//! so these tests need the runtime library but no attached device. Each test
//! applies a program and reads the native state back to check that the bridge
//! writes the same layout DSView's trigger dock writes.

use std::sync::{Mutex, MutexGuard, OnceLock};

use dsview_sys::{
    RuntimeBridge, RuntimeError, TRIGGER_PROBE_CAPACITY, TRIGGER_STAGE_CAPACITY,
    TriggerObservation, TriggerProgram, TriggerProgramMode, TriggerStageProgram,
    source_runtime_library_path,
};

const SIMPLE_TRIGGER: u16 = 0;
const ADV_TRIGGER: u16 = 1;
const SERIAL_TRIGGER: u16 = 2;

fn runtime_test_guard() -> MutexGuard<'static, ()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn load_runtime() -> Option<RuntimeBridge> {
    let Some(path) = source_runtime_library_path() else {
        eprintln!("skipping trigger boundary test: no source-built runtime library");
        return None;
    };
    Some(RuntimeBridge::load(path).expect("source-built runtime library loads"))
}

fn pattern(conditions: &[(usize, u8)]) -> [u8; TRIGGER_PROBE_CAPACITY] {
    let mut values = [b'X'; TRIGGER_PROBE_CAPACITY];
    for &(probe, condition) in conditions {
        values[probe] = condition;
    }
    values
}

fn simple_program(conditions: &[(usize, u8)], position_percent: u16) -> TriggerProgram {
    TriggerProgram {
        mode: TriggerProgramMode::Simple,
        position_percent,
        stage_selector: 0,
        simple: pattern(conditions),
        stages: Vec::new(),
    }
}

#[test]
fn simple_trigger_program_writes_the_simple_row() {
    let _guard = runtime_test_guard();
    let Some(runtime) = load_runtime() else {
        return;
    };

    runtime
        .apply_trigger(&simple_program(&[(0, b'R'), (3, b'1'), (15, b'C')], 10))
        .expect("simple trigger applies");
    let state = runtime.trigger_state().expect("trigger state reads back");

    assert!(state.enabled);
    assert_eq!(state.mode, SIMPLE_TRIGGER);
    assert_eq!(state.position_percent, 10);
    assert_eq!(state.stage_selector, 0);
    assert_eq!(state.simple0, pattern(&[(0, b'R'), (3, b'1'), (15, b'C')]));
    assert_eq!(state.simple1, pattern(&[]));
    assert_eq!(state.stages.len(), TRIGGER_STAGE_CAPACITY);
    for stage in &state.stages {
        assert_eq!(*stage, TriggerStageProgram::dont_care());
    }
}

#[test]
fn stage_trigger_program_maps_patterns_by_probe_index() {
    let _guard = runtime_test_guard();
    let Some(runtime) = load_runtime() else {
        return;
    };

    let first = TriggerStageProgram {
        value0: pattern(&[(0, b'R'), (15, b'0')]),
        value1: pattern(&[(5, b'F')]),
        logic: 0b10,
        inv0: false,
        inv1: true,
        count0: 3,
        count1: 0,
    };
    let second = TriggerStageProgram {
        value0: pattern(&[(7, b'1'), (8, b'0')]),
        value1: pattern(&[]),
        logic: 0b01,
        inv0: true,
        inv1: false,
        count0: 1,
        count1: 0,
    };
    let program = TriggerProgram {
        mode: TriggerProgramMode::Stage,
        position_percent: 50,
        stage_selector: 1,
        simple: pattern(&[]),
        stages: vec![first, second],
    };

    runtime
        .apply_trigger(&program)
        .expect("stage trigger applies");
    let state = runtime.trigger_state().expect("trigger state reads back");

    assert!(state.enabled);
    assert_eq!(state.mode, ADV_TRIGGER);
    assert_eq!(state.position_percent, 50);
    assert_eq!(state.stage_selector, 1);
    assert_eq!(state.stages[0], first);
    assert_eq!(state.stages[1], second);
    for stage in &state.stages[2..] {
        assert_eq!(*stage, TriggerStageProgram::dont_care());
    }
}

#[test]
fn serial_trigger_program_sets_serial_mode() {
    let _guard = runtime_test_guard();
    let Some(runtime) = load_runtime() else {
        return;
    };

    let mut stages = vec![TriggerStageProgram::dont_care(); 4];
    stages[0].value0 = pattern(&[(1, b'0')]);
    stages[0].value1 = pattern(&[(1, b'1')]);
    stages[1].value0 = pattern(&[(0, b'R')]);
    stages[1].count0 = 1;
    stages[2].value0 = pattern(&[(2, b'0')]);
    stages[3].value0 = pattern(&[(0, b'1'), (1, b'0'), (2, b'1'), (3, b'0')]);
    stages[3].count0 = 3;
    let program = TriggerProgram {
        mode: TriggerProgramMode::Serial,
        position_percent: 1,
        stage_selector: 0,
        simple: pattern(&[]),
        stages: stages.clone(),
    };

    runtime
        .apply_trigger(&program)
        .expect("serial trigger applies");
    let state = runtime.trigger_state().expect("trigger state reads back");

    assert!(state.enabled);
    assert_eq!(state.mode, SERIAL_TRIGGER);
    assert_eq!(state.stage_selector, 0);
    assert_eq!(&state.stages[..4], stages.as_slice());
}

#[test]
fn disable_trigger_restores_the_dsview_defaults() {
    let _guard = runtime_test_guard();
    let Some(runtime) = load_runtime() else {
        return;
    };

    runtime
        .apply_trigger(&simple_program(&[(2, b'F')], 20))
        .expect("simple trigger applies");
    runtime.disable_trigger().expect("trigger disables");
    let state = runtime.trigger_state().expect("trigger state reads back");

    assert!(!state.enabled);
    assert_eq!(state.mode, SIMPLE_TRIGGER);
    assert_eq!(state.position_percent, 0);
    assert_eq!(state.stage_selector, 0);
    assert_eq!(state.simple0, pattern(&[]));
    for stage in &state.stages {
        assert_eq!(*stage, TriggerStageProgram::dont_care());
    }
}

#[test]
fn bridge_rejects_programs_that_would_trip_dsview_asserts() {
    let _guard = runtime_test_guard();
    let Some(runtime) = load_runtime() else {
        return;
    };

    runtime
        .apply_trigger(&simple_program(&[(4, b'1')], 5))
        .expect("baseline trigger applies");

    let invalid_programs = [
        simple_program(&[(0, b'Z')], 5),
        simple_program(&[(0, b'R')], 101),
        TriggerProgram {
            mode: TriggerProgramMode::Stage,
            position_percent: 5,
            stage_selector: 0,
            simple: pattern(&[]),
            stages: Vec::new(),
        },
        TriggerProgram {
            mode: TriggerProgramMode::Stage,
            position_percent: 5,
            stage_selector: TRIGGER_STAGE_CAPACITY as u16,
            simple: pattern(&[]),
            stages: vec![TriggerStageProgram::dont_care()],
        },
        TriggerProgram {
            mode: TriggerProgramMode::Stage,
            position_percent: 5,
            stage_selector: 0,
            simple: pattern(&[]),
            stages: vec![TriggerStageProgram::dont_care(); TRIGGER_STAGE_CAPACITY + 1],
        },
    ];

    for program in &invalid_programs {
        let error = runtime
            .apply_trigger(program)
            .expect_err("invalid program is rejected");
        assert!(
            matches!(error, RuntimeError::InvalidArgument(_)),
            "unexpected error {error:?}"
        );
    }

    // Rejected programs never reach the native setters.
    let state = runtime.trigger_state().expect("trigger state reads back");
    assert!(state.enabled);
    assert_eq!(state.position_percent, 5);
    assert_eq!(state.simple0, pattern(&[(4, b'1')]));
}

#[test]
fn trigger_observation_is_empty_before_any_acquisition() {
    let _guard = runtime_test_guard();
    let Some(runtime) = load_runtime() else {
        return;
    };

    runtime
        .reset_acquisition_summary()
        .expect("acquisition summary resets");
    let observation = runtime
        .trigger_observation()
        .expect("trigger observation reads");
    assert_eq!(observation, TriggerObservation::default());
    assert!(!observation.fired());
}
