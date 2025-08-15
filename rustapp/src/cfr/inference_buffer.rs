use tokio::time::Duration;
use std::collections::VecDeque;

// makes like a queue to handle queries to value handler
// I want this to be async

// makes a queue that stores input variables for prediction
// connects to value_function for inference
// sends results to updater
// so we update mixed strategy after every iteration
struct Query {
    // [Placeholder]
    data: Vec<f64>,
    // Some path updating metadata
}
// Will prob needs double buffers, one where adding gets done, another where removing gets done
// Then swapping when the processes are complete
struct InferenceBuffer {
    max_batch_size: usize,
    max_time_interval: Duration,
    queue: VecDeque<Query>
}

impl InferenceBuffer {
    fn new(max_batch_size: usize, max_time_interval: Duration) -> Self {
        InferenceBuffer {
            max_batch_size,
            max_time_interval,
            queue: VecDeque::new(),
        }
    }
}

