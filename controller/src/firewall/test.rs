use axum::{
    body::Body,
    extract::Path,
    http::{Request, StatusCode},
    response::Response,
};
use chrono::NaiveDateTime;
use message::firewall::{self, LogKind, RuleChange, Status, StoredEventDecoded, StoredRuleDecoded};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;

use crate::{router, AppState, Socket};

// Mock Socket implementation for testing
#[derive(Debug)]
struct MockSocket {
    responses: Vec<firewall::Response>,
    sent_messages: Arc<Mutex<Vec<message::Message>>>,
}

impl MockSocket {
    fn new(responses: Vec<firewall::Response>) -> Self {
        Self {
            responses,
            sent_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn get_sent_messages(&self) -> Vec<message::Message> {
        self.sent_messages.lock().await.clone()
    }
}

#[tokio::test]
async fn test_status_endpoint() {
    // Setup
    let mock_socket = MockSocket::new(vec![firewall::Response::Status(Status::Running)]);
    let app_state = AppState {
        firewall_pool: Pool::builder(mock_socket).max_size(1).build().unwrap(),
    };
    
    let app = router().with_state(app_state);
    
    // Test
    let response = app
        .oneshot(
            Request::builder()
                .uri("/state")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let status: Status = serde_json::from_slice(&body).unwrap();
    assert_eq!(status, Status::Running);
}

#[tokio::test]
async fn test_get_rules() {
    // Setup
    let mock_rules = vec![
        StoredRuleDecoded {
            id: 1,
            name: "Test Rule 1".to_string(),
            enabled: true,
            // ... other fields ...
        },
        StoredRuleDecoded {
            id: 2,
            name: "Test Rule 2".to_string(),
            enabled: false,
            // ... other fields ...
        },
    ];
    
    let mock_socket = MockSocket::new(vec![firewall::Response::Rules(mock_rules.clone())]);
    let app_state = AppState {
        firewall_pool: Pool::builder(mock_socket).max_size(1).build().unwrap(),
    };
    
    let app = router().with_state(app_state);
    
    // Test
    let response = app
        .oneshot(
            Request::builder()
                .uri("/rules")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let rules: Vec<StoredRuleDecoded> = serde_json::from_slice(&body).unwrap();
    assert_eq!(rules, mock_rules);
}

#[tokio::test]
async fn test_toggle_rule() {
    // Setup
    let rule_id = 1;
    let mock_socket = MockSocket::new(vec![
        firewall::Response::RuleChange(RuleChange::Change(firewall::RuleStatus::Active))
    ]);
    let app_state = AppState {
        firewall_pool: Pool::builder(mock_socket).max_size(1).build().unwrap(),
    };
    
    let app = router().with_state(app_state);
    
    // Test
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/rules/{}/toggle", rule_id))
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_query_events() {
    // Setup
    let mock_events = vec![
        StoredEventDecoded {
            timestamp: NaiveDateTime::from_timestamp_opt(1635724800, 0).unwrap(),
            kind: LogKind::PacketAccepted { rule_id: 1 },
        },
        StoredEventDecoded {
            timestamp: NaiveDateTime::from_timestamp_opt(1635724801, 0).unwrap(),
            kind: LogKind::PacketDropped { rule_id: 2 },
        },
    ];
    
    let mock_socket = MockSocket::new(vec![firewall::Response::Events(mock_events.clone())]);
    let app_state = AppState {
        firewall_pool: Pool::builder(mock_socket).max_size(1).build().unwrap(),
    };
    
    let app = router().with_state(app_state);
    
    // Test
    let query = EventQuery {
        start_time: None,
        end_time: None,
        limit: Some(10),
    };
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/events/query")
                .method("GET")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&query).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let events: Vec<StoredEventDecoded> = serde_json::from_slice(&body).unwrap();
    assert_eq!(events, mock_events);
}

// Implementation of the MockSocket for the Manager trait
impl deadpool::managed::Manager for MockSocket {
    type Type = Self;
    type Error = ();

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(self.clone())
    }

    async fn recycle(
        &self,
        _obj: &mut Self::Type,
        _metrics: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        Ok(())
    }
}

// Implement required methods for MockSocket to match the real Socket
impl MockSocket {
    async fn send(&mut self, msg: message::Message) {
        self.sent_messages.lock().await.push(msg);
    }

    async fn read(&mut self) -> firewall::Response {
        self.responses.remove(0)
    }

    async fn status(&mut self) -> firewall::Status {
        match self.read().await {
            firewall::Response::Status(status) => status,
            _ => panic!("Unexpected response type"),
        }
    }
    
    async fn get_rules(&mut self) -> Vec<StoredRuleDecoded> {
        match self.read().await {
            firewall::Response::Rules(rules) => rules,
            _ => panic!("Unexpected response type"),
        }
    }
    
    async fn toggle(&mut self, idx: u32) -> firewall::RuleChange {
        self.send(message::Message::Firewall(firewall::Request::ToggleRule(idx)))
            .await;
        match self.read().await {
            firewall::Response::RuleChange(change) => change,
            _ => panic!("Unexpected response type"),
        }
    }
    
    async fn get_events(&mut self, query: EventQuery) -> Vec<StoredEventDecoded> {
        self.send(message::Message::Firewall(firewall::Request::GetEvents(query)))
            .await;
        match self.read().await {
            firewall::Response::Events(events) => events,
            _ => panic!("Unexpected response type"),
        }
    }
}
