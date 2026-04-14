// 请求上下文 — 直接使用 fbc-starter 统一提供的 RequestContext
//
// fbc-starter 的 user_context_middleware 已在 Server 启动时自动注册，
// 会从网关透传的 X-User-Id / X-Tenant-Id / X-Username Header 中解析用户信息。

pub use fbc_starter::RequestContext;
