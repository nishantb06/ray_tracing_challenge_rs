use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, Mutex};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentEvent {
    pub run_id: String,
    pub iteration: u32,
    pub phase: String,
    pub payload: serde_json::Value,
    pub ts: String,
}

#[derive(Clone, Default)]
pub struct AgentBus {
    runs: Arc<Mutex<HashMap<String, RunEvents>>>,
}
struct RunEvents {
    events: Vec<AgentEvent>,
    sender: broadcast::Sender<AgentEvent>,
}
impl AgentBus {
    pub async fn ingest(&self, event: AgentEvent) {
        let mut runs = self.runs.lock().await;
        let state = runs.entry(event.run_id.clone()).or_insert_with(|| {
            let (sender, _) = broadcast::channel(64);
            RunEvents { events: Vec::new(), sender }
        });
        state.events.push(event.clone());
        let _ = state.sender.send(event);
    }
    pub async fn snapshot_and_subscribe(&self, run_id: &str) -> (Vec<AgentEvent>, broadcast::Receiver<AgentEvent>) {
        let mut runs = self.runs.lock().await;
        let state = runs.entry(run_id.to_owned()).or_insert_with(|| {
            let (sender, _) = broadcast::channel(64);
            RunEvents { events: Vec::new(), sender }
        });
        (state.events.clone(), state.sender.subscribe())
    }
}
