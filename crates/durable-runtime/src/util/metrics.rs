use metrics::Gauge;

/// A span that increments a gauge upon being entered and decrements it on exit.
pub(crate) struct MetricSpan {
    gauge: Gauge,
}

impl MetricSpan {
    pub fn enter(gauge: Gauge) -> Self {
        gauge.increment(1);

        Self { gauge }
    }
}

impl Drop for MetricSpan {
    fn drop(&mut self) {
        self.gauge.decrement(1);
    }
}
