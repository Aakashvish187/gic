//! Unit tests for Kubernetes apiVersion Evaluation.

use crate::kubernetes::api_version::{ApiVersionStatus, K8sApiVersionEvaluator};
use crate::kubernetes::resource_detector::K8sResourceKind;

#[test]
fn test_api_version_evaluation() {
    let evaluator = K8sApiVersionEvaluator::new();

    assert_eq!(
        evaluator.evaluate(K8sResourceKind::Deployment, "apps/v1"),
        ApiVersionStatus::Valid
    );

    assert!(matches!(
        evaluator.evaluate(K8sResourceKind::Deployment, "extensions/v1beta1"),
        ApiVersionStatus::Deprecated { .. }
    ));

    assert!(matches!(
        evaluator.evaluate(K8sResourceKind::Pod, "apps/v1"),
        ApiVersionStatus::Invalid { .. }
    ));
}
