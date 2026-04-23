//! # 通用状态机
//!
//! 为任务流转、订单状态管理等场景提供通用的状态机抽象。
//!
//! ## 使用示例
//!
//! ```rust
//! use fbc_starter::state_machine::{StateMachine, SimpleStateMachine};
//!
//! #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//! enum OrderStatus { Created, Paid, Shipped, Done, Cancelled }
//!
//! let sm = SimpleStateMachine::new(vec![
//!     (OrderStatus::Created, OrderStatus::Paid),
//!     (OrderStatus::Created, OrderStatus::Cancelled),
//!     (OrderStatus::Paid,    OrderStatus::Shipped),
//!     (OrderStatus::Shipped, OrderStatus::Done),
//! ]);
//!
//! assert!(sm.can_transition(&OrderStatus::Created, &OrderStatus::Paid));
//! assert!(!sm.can_transition(&OrderStatus::Done, &OrderStatus::Created));
//! ```

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;

/// 状态机 trait — 定义状态转换规则
///
/// 泛型参数 `S` 为状态枚举类型
pub trait StateMachine<S>: Send + Sync
where
    S: Clone + PartialEq + Eq + Hash + Send + Sync,
{
    /// 检查从 `from` 到 `to` 的状态转换是否合法
    fn can_transition(&self, from: &S, to: &S) -> bool;

    /// 获取从指定状态可以转换到的所有目标状态
    fn next_states(&self, from: &S) -> Vec<S>;

    /// 执行状态转换，不合法时返回错误
    fn transition(&self, from: &S, to: &S) -> Result<(), StateMachineError<S>>
    where
        S: fmt::Debug,
    {
        if self.can_transition(from, to) {
            Ok(())
        } else {
            Err(StateMachineError::IllegalTransition {
                from: from.clone(),
                to: to.clone(),
            })
        }
    }
}

/// 状态机错误
#[derive(Debug, thiserror::Error)]
pub enum StateMachineError<S: fmt::Debug> {
    /// 非法的状态转换
    #[error("非法的状态转换: {from:?} → {to:?}")]
    IllegalTransition { from: S, to: S },
}

/// 简单状态机 — 基于转换对列表的通用实现
///
/// 只需提供合法的 `(from, to)` 转换对即可使用。
/// 内部使用 `HashMap<S, HashSet<S>>` 索引，查询复杂度 O(1)。
pub struct SimpleStateMachine<S: Clone + PartialEq + Eq + Hash> {
    /// 转换表：from → 可达的 to 集合
    transitions: HashMap<S, HashSet<S>>,
}

impl<S> SimpleStateMachine<S>
where
    S: Clone + PartialEq + Eq + Hash,
{
    /// 创建状态机
    ///
    /// # 参数
    /// - `pairs`: 合法的状态转换对列表 `[(from, to), ...]`
    pub fn new(pairs: Vec<(S, S)>) -> Self {
        let mut transitions: HashMap<S, HashSet<S>> = HashMap::new();
        for (from, to) in pairs {
            transitions
                .entry(from)
                .or_insert_with(HashSet::new)
                .insert(to);
        }
        Self { transitions }
    }
}

impl<S> StateMachine<S> for SimpleStateMachine<S>
where
    S: Clone + PartialEq + Eq + Hash + Send + Sync,
{
    fn can_transition(&self, from: &S, to: &S) -> bool {
        self.transitions
            .get(from)
            .map(|targets| targets.contains(to))
            .unwrap_or(false)
    }

    fn next_states(&self, from: &S) -> Vec<S> {
        self.transitions
            .get(from)
            .map(|targets| targets.iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum TestStatus {
        Init,
        Processing,
        Done,
        Failed,
    }

    fn create_test_sm() -> SimpleStateMachine<TestStatus> {
        SimpleStateMachine::new(vec![
            (TestStatus::Init, TestStatus::Processing),
            (TestStatus::Processing, TestStatus::Done),
            (TestStatus::Processing, TestStatus::Failed),
            (TestStatus::Failed, TestStatus::Init), // 重试
        ])
    }

    #[test]
    fn test_valid_transitions() {
        let sm = create_test_sm();
        assert!(sm.can_transition(&TestStatus::Init, &TestStatus::Processing));
        assert!(sm.can_transition(&TestStatus::Processing, &TestStatus::Done));
        assert!(sm.can_transition(&TestStatus::Processing, &TestStatus::Failed));
        assert!(sm.can_transition(&TestStatus::Failed, &TestStatus::Init));
    }

    #[test]
    fn test_invalid_transitions() {
        let sm = create_test_sm();
        assert!(!sm.can_transition(&TestStatus::Init, &TestStatus::Done));
        assert!(!sm.can_transition(&TestStatus::Done, &TestStatus::Init));
        assert!(!sm.can_transition(&TestStatus::Done, &TestStatus::Processing));
        assert!(!sm.can_transition(&TestStatus::Init, &TestStatus::Failed));
    }

    #[test]
    fn test_next_states() {
        let sm = create_test_sm();
        let next = sm.next_states(&TestStatus::Processing);
        assert_eq!(next.len(), 2);
        assert!(next.contains(&TestStatus::Done));
        assert!(next.contains(&TestStatus::Failed));
    }

    #[test]
    fn test_transition_ok() {
        let sm = create_test_sm();
        assert!(sm.transition(&TestStatus::Init, &TestStatus::Processing).is_ok());
    }

    #[test]
    fn test_transition_err() {
        let sm = create_test_sm();
        let result = sm.transition(&TestStatus::Done, &TestStatus::Init);
        assert!(result.is_err());
    }
}
