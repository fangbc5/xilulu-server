mod client;
pub mod circuit_breaker;
mod discovery;
pub mod health;
mod load_balancer;
pub mod resilient_client;

pub use client::{create_grpc_channel, create_grpc_client, GrpcClientBuilder};
pub use circuit_breaker::{
    get_circuit_breaker, get_existing_circuit_breaker, get_existing_instance_circuit_breaker,
    get_instance_circuit_breaker, CircuitBreaker, CircuitBreakerConfig, CircuitState,
};
pub use discovery::{get_service_endpoints, ServiceEndpoint};
pub use health::{
    get_health_status, get_service_health_snapshot, is_available, mark_healthy, record_healthy,
    record_unhealthy, remove_instance, start_health_checker, HealthCheckConfig, HealthStatus,
};
pub use load_balancer::{get_load_balancer, LoadBalancer, RoundRobinLoadBalancer};
pub use resilient_client::{grpc_call, ResilientGrpcClient, RetryConfig};
