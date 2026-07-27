//! Unit tests for Workload resources (Deployment, StatefulSet, DaemonSet, Job, CronJob).

use crate::kubernetes::engine::K8sEngine;

#[test]
fn test_deployment_validation_missing_selector() {
    let source = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: broken-deploy
spec:
  replicas: 3
"#;

    let engine = K8sEngine::default();
    let (diags, _) = engine.validate(source);

    let sel_diag = diags
        .iter()
        .find(|d| d.rule_id == "k8s-deployment-missing-selector");
    assert!(sel_diag.is_some());
}

#[test]
fn test_job_invalid_restart_policy() {
    let source = r#"
apiVersion: batch/v1
kind: Job
metadata:
  name: bad-job
spec:
  template:
    spec:
      restartPolicy: Always
"#;

    let engine = K8sEngine::default();
    let (diags, _) = engine.validate(source);

    let policy_diag = diags
        .iter()
        .find(|d| d.rule_id == "k8s-job-invalid-restart-policy");
    assert!(policy_diag.is_some());
}
