//! Performance and benchmark unit tests.

use crate::linux::engine::{LinuxEngine, LinuxEngineOptions};

#[test]
fn test_large_bash_script_performance() {
    let mut source = String::from("#!/bin/bash\n");
    for i in 0..1000 {
        source.push_str(&format!("echo 'Line {}'\n", i));
    }

    let engine = LinuxEngine::new(LinuxEngineOptions {
        enable_cache: true,
        cache_capacity: 100,
    });
    let (diags, _) = engine.validate_bash(&source);

    assert_eq!(engine.cache().len(), 1);

    // Cache hit
    let (diags2, _) = engine.validate_bash(&source);
    assert_eq!(diags, diags2);
}
