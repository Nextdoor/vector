use std::collections::BTreeSet;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use value::Value;
use vector_core::{
    config::{DataType, Input, Output},
    event::{
        metric::{MetricData, Sample},
        Event, MetricValue,
    },
    schema,
    transform::{FunctionTransform, OutputBuffer, Transform, TransformConfig, TransformContext},
};

use crate::config::TransformDescription;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BasicTransformConfig {
    suffix: String,
    increase: f64,
}

impl_generate_config_from_default!(BasicTransformConfig);

inventory::submit! {
    TransformDescription::new::<BasicTransformConfig>("basic_transform")
}

impl BasicTransformConfig {
    pub const fn new(suffix: String, increase: f64) -> Self {
        Self { suffix, increase }
    }
}

#[async_trait]
#[typetag::serde(name = "basic_transform")]
impl TransformConfig for BasicTransformConfig {
    async fn build(&self, _globals: &TransformContext) -> crate::Result<Transform> {
        Ok(Transform::function(BasicTransform {
            suffix: self.suffix.clone(),
            increase: self.increase,
        }))
    }

    fn input(&self) -> Input {
        Input::all()
    }

    fn outputs(&self, _: &schema::Definition) -> Vec<Output> {
        vec![Output::default(DataType::all())]
    }

    fn transform_type(&self) -> &'static str {
        "basic_transform"
    }
}

#[derive(Clone, Debug)]
struct BasicTransform {
    suffix: String,
    increase: f64,
}

impl FunctionTransform for BasicTransform {
    fn transform(&mut self, output: &mut OutputBuffer, mut event: Event) {
        match &mut event {
            Event::Log(log) => {
                let mut v = log
                    .get(crate::config::log_schema().message_key())
                    .unwrap()
                    .to_string_lossy();
                v.push_str(&self.suffix);
                log.insert(crate::config::log_schema().message_key(), Value::from(v));
            }
            Event::Metric(metric) => {
                let increment = match metric.value() {
                    MetricValue::Counter { .. } => Some(MetricValue::Counter {
                        value: self.increase,
                    }),
                    MetricValue::Gauge { .. } => Some(MetricValue::Gauge {
                        value: self.increase,
                    }),
                    MetricValue::Distribution { statistic, .. } => {
                        Some(MetricValue::Distribution {
                            samples: vec![Sample {
                                value: self.increase,
                                rate: 1,
                            }],
                            statistic: *statistic,
                        })
                    }
                    MetricValue::AggregatedHistogram { .. } => None,
                    MetricValue::AggregatedSummary { .. } => None,
                    MetricValue::Sketch { .. } => None,
                    MetricValue::Set { .. } => {
                        let mut values = BTreeSet::new();
                        values.insert(self.suffix.clone());
                        Some(MetricValue::Set { values })
                    }
                };
                if let Some(increment) = increment {
                    assert!(metric.add(&MetricData {
                        kind: metric.kind(),
                        timestamp: metric.timestamp(),
                        value: increment,
                    }));
                }
            }
            Event::Trace(trace) => {
                let mut v = trace
                    .get(crate::config::log_schema().message_key())
                    .unwrap()
                    .to_string_lossy();
                v.push_str(&self.suffix);
                trace.insert(crate::config::log_schema().message_key(), Value::from(v));
            }
        };
        output.push(event);
    }
}
