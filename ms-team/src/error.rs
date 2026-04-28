//! 组织服务统一错误处理模块
//!
//! 本模块定义了组织服务（ms-team）的所有错误类型和错误码。
//!
//! ## 错误码范围（6000-6999）
//!
//! - 6000-6099: 组织模块错误
//! - 6100-6199: 部门模块错误
//! - 6200-6299: 岗位模块错误
//! - 6300-6399: 员工模块错误
//! - 6400-6449: 员工-部门关系错误
//! - 6450-6499: 员工-岗位关系错误
//! - 6500-6599: 数据库与系统错误
//! - 6600-6699: 参数验证错误
//! - 6700-6799: 业务逻辑错误
//! - 6800-6849: 权限与认证错误
//! - 6850-6899: 通讯录模块错误
//! - 6900-6999: 系统错误
//!
//! ## 使用示例
//!
//! ```ignore
//! use OrganizationError;
//!
//! // 在 Handler 中使用
//! async fn get_organization(id: i64) -> Result<Json<OrganizationDto>, OrganizationError> {
//!     let org = repository
//!         .find_by_id(id)
//!         .await?
//!         .ok_or(OrganizationError::OrganizationNotFound)?;
//!     Ok(Json(org))
//! }
//!
//! // 返回带上下文的错误
//! if code.is_empty() {
//!     return Err(OrganizationError::OrganizationCodeInvalid(code.clone()));
//! }
//! ```
//!
//! ## HTTP 状态码映射
//!
//! - 400 Bad Request: 参数验证错误
//! - 401 Unauthorized: 认证失败
//! - 403 Forbidden: 无权限操作
//! - 404 Not Found: 资源不存在
//! - 409 Conflict: 业务冲突（重复、状态冲突等）
//! - 500 Internal Server Error: 其他错误
//! - 503 Service Unavailable: 服务不可用
//!

/// 组织服务错误码（范围: 6000-6999）
pub mod error_code {
    // ============ 组织模块 (6000-6099) ============
    /// 6001 组织不存在
    pub const ORGANIZATION_NOT_FOUND: i32 = 6001;
    /// 6002 组织已存在
    pub const ORGANIZATION_EXISTS: i32 = 6002;
    /// 6003 组织代码已存在
    pub const ORGANIZATION_CODE_DUP: i32 = 6003;
    /// 6004 组织名称已存在
    pub const ORGANIZATION_NAME_DUP: i32 = 6004;
    /// 6005 组织有下级组织，无法删除
    pub const ORGANIZATION_HAS_CHILDREN: i32 = 6005;
    /// 6006 组织有部门，无法删除
    pub const ORGANIZATION_HAS_DEPT: i32 = 6006;
    /// 6007 组织代码格式错误（需要大写字母和数字的组合）
    pub const ORGANIZATION_CODE_INVALID: i32 = 6007;
    /// 6008 组织名称不能为空
    pub const ORGANIZATION_NAME_EMPTY: i32 = 6008;
    /// 6009 无权限操作该组织
    pub const ORGANIZATION_UNAUTHORIZED: i32 = 6009;
    /// 6010 组织状态非法
    pub const ORGANIZATION_STATUS_INVALID: i32 = 6010;

    // ============ 部门模块 (6100-6199) ============
    /// 6101 部门不存在
    pub const DEPARTMENT_NOT_FOUND: i32 = 6101;
    /// 6102 部门已存在
    pub const DEPARTMENT_EXISTS: i32 = 6102;
    /// 6103 部门代码已存在
    pub const DEPARTMENT_CODE_DUP: i32 = 6103;
    /// 6104 部门有下级部门，无法删除
    pub const DEPARTMENT_HAS_CHILDREN: i32 = 6104;
    /// 6105 部门有员工，无法删除
    pub const DEPARTMENT_HAS_EMPLOYEES: i32 = 6105;
    /// 6106 部门代码格式错误
    pub const DEPARTMENT_CODE_INVALID: i32 = 6106;
    /// 6107 部门名称不能为空
    pub const DEPARTMENT_NAME_EMPTY: i32 = 6107;
    /// 6108 上级部门不存在
    pub const DEPARTMENT_PARENT_NOT_FOUND: i32 = 6108;
    /// 6109 无法设置自己为上级部门
    pub const DEPARTMENT_PARENT_SELF: i32 = 6109;
    /// 6110 无权限操作此部门
    pub const DEPARTMENT_UNAUTHORIZED: i32 = 6110;
    /// 6111 部门层级过深（超过10级限制）
    pub const DEPARTMENT_LEVEL_TOO_DEEP: i32 = 6111;
    /// 6112 部门状态非法
    pub const DEPARTMENT_STATUS_INVALID: i32 = 6112;

    // ============ 岗位/职位模块 (6200-6299) ============
    /// 6201 岗位不存在
    pub const POSITION_NOT_FOUND: i32 = 6201;
    /// 6202 岗位已存在
    pub const POSITION_EXISTS: i32 = 6202;
    /// 6203 岗位代码已存在
    pub const POSITION_CODE_DUP: i32 = 6203;
    /// 6204 岗位有员工，无法删除
    pub const POSITION_HAS_EMPLOYEES: i32 = 6204;
    /// 6205 岗位名称为空
    pub const POSITION_NAME_EMPTY: i32 = 6205;
    /// 6206 岗位代码格式错误
    pub const POSITION_CODE_INVALID: i32 = 6206;
    /// 6207 岗位等级不合法
    pub const POSITION_LEVEL_INVALID: i32 = 6207;
    /// 6208 无权限操作此岗位
    pub const POSITION_UNAUTHORIZED: i32 = 6208;
    /// 6209 岗位状态非法
    pub const POSITION_STATUS_INVALID: i32 = 6209;

    // ============ 员工模块 (6300-6399) ============
    /// 6301 员工不存在
    pub const EMPLOYEE_NOT_FOUND: i32 = 6301;
    /// 6302 员工已存在
    pub const EMPLOYEE_EXISTS: i32 = 6302;
    /// 6303 员工工号已存在
    pub const EMPLOYEE_NO_EXISTS: i32 = 6303;
    /// 6304 用户已是该组织员工
    pub const USER_ALREADY_EMPLOYEE: i32 = 6304;
    /// 6305 员工工号为空
    pub const EMPLOYEE_NO_EMPTY: i32 = 6305;
    /// 6306 员工名称为空
    pub const EMPLOYEE_NAME_EMPTY: i32 = 6306;
    /// 6307 员工工号格式错误
    pub const EMPLOYEE_NO_INVALID: i32 = 6307;
    /// 6308 员工邮箱格式错误
    pub const EMPLOYEE_EMAIL_INVALID: i32 = 6308;
    /// 6309 员工手机号格式错误
    pub const EMPLOYEE_PHONE_INVALID: i32 = 6309;
    /// 6310 员工身份证号格式错误
    pub const EMPLOYEE_ID_CARD_INVALID: i32 = 6310;
    /// 6311 无权限操作此员工
    pub const EMPLOYEE_UNAUTHORIZED: i32 = 6311;
    /// 6312 员工状态非法
    pub const EMPLOYEE_STATUS_INVALID: i32 = 6312;
    /// 6313 员工至少需要关联一个部门
    pub const EMPLOYEE_NO_DEPT: i32 = 6313;

    // ============ 员工-部门关系 (6400-6449) ============
    /// 6401 员工部门关系不存在
    pub const EMPLOYEE_DEPT_REL_NOT_FOUND: i32 = 6401;
    /// 6402 员工部门关系已存在
    pub const EMPLOYEE_DEPT_REL_EXISTS: i32 = 6402;
    /// 6403 员工不能移出最后一个部门
    pub const EMPLOYEE_DEPT_LAST: i32 = 6403;
    /// 6404 员工在此部门下没有岗位
    pub const EMPLOYEE_DEPT_NO_POSITION: i32 = 6404;

    // ============ 员工-岗位关系 (6450-6499) ============
    /// 6451 员工岗位关系不存在
    pub const EMPLOYEE_POSITION_REL_NOT_FOUND: i32 = 6451;
    /// 6452 员工岗位关系已存在
    pub const EMPLOYEE_POSITION_REL_EXISTS: i32 = 6452;
    /// 6453 员工不能移出最后一个岗位
    pub const EMPLOYEE_POSITION_LAST: i32 = 6453;

    // ============ 数据库与系统错误 (6500-6599) ============
    /// 6501 数据库连接失败
    pub const DATABASE_CONNECT_ERROR: i32 = 6501;
    /// 6502 数据库操作失败
    pub const DATABASE_QUERY_ERROR: i32 = 6502;
    /// 6503 事务操作失败
    pub const DATABASE_TRANSACTION_ERROR: i32 = 6503;
    /// 6504 并发冲突（乐观锁）
    pub const CONCURRENT_CONFLICT: i32 = 6504;
    /// 6505 数据一致性检查失败
    pub const DATA_CONSISTENCY_ERROR: i32 = 6505;

    // ============ 参数验证错误 (6600-6699) ============
    /// 6601 参数为空
    pub const PARAM_NULL: i32 = 6601;
    /// 6602 参数类型错误
    pub const PARAM_TYPE_ERROR: i32 = 6602;
    /// 6603 参数值超出范围
    pub const PARAM_VALUE_OUT_OF_RANGE: i32 = 6603;
    /// 6604 参数格式错误
    pub const PARAM_FORMAT_ERROR: i32 = 6604;
    /// 6605 必填参数缺失
    pub const PARAM_REQUIRED: i32 = 6605;
    /// 6606 分页参数错误（页码或大小非法）
    pub const PARAM_PAGINATION_ERROR: i32 = 6606;
    /// 6607 排序参数错误
    pub const PARAM_ORDER_ERROR: i32 = 6607;
    /// 6608 时间参数错误
    pub const PARAM_DATE_ERROR: i32 = 6608;

    // ============ 业务逻辑错误 (6700-6799) ============
    /// 6701 业务规则冲突
    pub const BUSINESS_CONFLICT: i32 = 6701;
    /// 6702 操作状态非法
    pub const BUSINESS_STATE_ERROR: i32 = 6702;
    /// 6703 批量操作部分失败
    pub const BUSINESS_PARTIAL_FAILURE: i32 = 6703;
    /// 6704 导出数据为空
    pub const BUSINESS_NO_DATA_TO_EXPORT: i32 = 6704;
    /// 6705 导入数据格式错误
    pub const BUSINESS_IMPORT_FORMAT_ERROR: i32 = 6705;
    /// 6706 导入数据验证失败
    pub const BUSINESS_IMPORT_VALIDATION_FAILED: i32 = 6706;

    // ============ 权限与认证错误 (6800-6899) ============
    /// 6801 无权限执行此操作
    pub const PERMISSION_DENIED: i32 = 6801;
    /// 6802 认证失败
    pub const UNAUTHORIZED: i32 = 6802;
    /// 6803 令牌过期
    pub const TOKEN_EXPIRED: i32 = 6803;
    /// 6804 令牌无效
    pub const TOKEN_INVALID: i32 = 6804;

    // ============ 通讯录模块 (6850-6899) ============
    /// 6851 搜索引擎错误
    pub const CONTACTS_SEARCH_FAILED: i32 = 6851;
    /// 6852 部门成员数超限（include_children 查询）
    pub const CONTACTS_MEMBER_COUNT_EXCEEDED: i32 = 6852;
    /// 6853 员工不可见（权限限制）
    pub const CONTACTS_EMPLOYEE_NOT_VISIBLE: i32 = 6853;
    /// 6854 部门不可见（权限限制）
    pub const CONTACTS_DEPARTMENT_NOT_VISIBLE: i32 = 6854;

    // ============ 系统错误 (6900-6999) ============
    /// 6901 系统内部错误
    pub const INTERNAL_SERVER_ERROR: i32 = 6901;
    /// 6902 服务暂时不可用
    pub const SERVICE_UNAVAILABLE: i32 = 6902;
    /// 6903 操作超时
    pub const OPERATION_TIMEOUT: i32 = 6903;
    /// 6904 资源冲突
    pub const RESOURCE_CONFLICT: i32 = 6904;
    /// 6905 缓存错误
    pub const CACHE_ERROR: i32 = 6905;
}

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use fbc_starter::R;
use thiserror::Error;

/// 组织服务错误枚举（企业级错误处理）
#[derive(Debug, Error)]
pub enum OrganizationError {
    // ============ 组织模块错误 ============
    #[error("组织不存在")]
    OrganizationNotFound,

    #[error("组织已存在")]
    OrganizationExists,

    #[error("组织代码 '{0}' 已存在")]
    OrganizationCodeDuplicate(String),

    #[error("组织名称 '{0}' 已存在")]
    OrganizationNameDuplicate(String),

    #[error("组织有下级组织，无法删除")]
    OrganizationHasChildren,

    #[error("组织有部门，无法删除")]
    OrganizationHasEmployees,

    #[error("组织代码格式错误：{0}")]
    OrganizationCodeInvalid(String),

    #[error("组织名称不能为空")]
    OrganizationNameEmpty,

    #[error("无权限操作该组织")]
    OrganizationUnauthorized,

    #[error("组织状态非法")]
    OrganizationStatusInvalid,

    // ============ 部门模块错误 ============
    #[error("部门不存在")]
    DepartmentNotFound,

    #[error("部门已存在：{0}")]
    DepartmentExists(String),

    #[error("部门代码 '{0}' 已存在")]
    DepartmentCodeDuplicate(String),

    #[error("部门有下级部门，无法删除")]
    DepartmentHasChildren,

    #[error("部门有员工，无法删除")]
    DepartmentHasEmployees,

    #[error("部门代码格式错误：{0}")]
    DepartmentCodeInvalid(String),

    #[error("部门名称不能为空")]
    DepartmentNameEmpty,

    #[error("上级部门不存在")]
    DepartmentParentNotFound,

    #[error("无法设置自己为上级部门")]
    DepartmentParentSelf,

    #[error("无权限操作此部门")]
    DepartmentUnauthorized,

    #[error("部门层级过深（超过10级限制）")]
    DepartmentLevelTooDeep,

    #[error("部门状态非法")]
    DepartmentStatusInvalid,

    // ============ 岗位模块错误 ============
    #[error("岗位不存在")]
    PositionNotFound,

    #[error("岗位已存在：{0}")]
    PositionExists(String),

    #[error("岗位代码 '{0}' 已存在")]
    PositionCodeDuplicate(String),

    #[error("岗位有员工，无法删除")]
    PositionHasEmployees,

    #[error("岗位名称不能为空")]
    PositionNameEmpty,

    #[error("岗位代码格式错误：{0}")]
    PositionCodeInvalid(String),

    #[error("岗位等级不合法")]
    PositionLevelInvalid,

    #[error("无权限操作此岗位")]
    PositionUnauthorized,

    #[error("岗位状态非法")]
    PositionStatusInvalid,

    // ============ 员工模块错误 ============
    #[error("员工不存在")]
    EmployeeNotFound,

    #[error("员工已存在")]
    EmployeeExists,

    #[error("员工工号 '{0}' 已存在")]
    EmployeeNoExists(String),

    #[error("用户已是该组织员工")]
    UserAlreadyEmployee,

    #[error("员工工号不能为空")]
    EmployeeNoEmpty,

    #[error("员工名称不能为空")]
    EmployeeNameEmpty,

    #[error("员工工号格式错误：{0}")]
    EmployeeNoInvalid(String),

    #[error("员工邮箱格式错误：{0}")]
    EmployeeEmailInvalid(String),

    #[error("员工手机号格式错误：{0}")]
    EmployeePhoneInvalid(String),

    #[error("员工身份证号格式错误")]
    EmployeeIdCardInvalid,

    #[error("无权限操作此员工")]
    EmployeeUnauthorized,

    #[error("员工状态非法")]
    EmployeeStatusInvalid,

    #[error("员工至少需要关联一个部门")]
    EmployeeNoDepartment,

    // ============ 员工-部门关系错误 ============
    #[error("员工部门关系不存在")]
    EmployeeDepartmentRelNotFound,

    #[error("员工部门关系已存在")]
    EmployeeDepartmentRelExists,

    #[error("员工不能移出最后一个部门")]
    EmployeeCannotRemoveLastDepartment,

    #[error("员工在此部门下没有岗位")]
    EmployeeNoDepartmentPosition,

    // ============ 员工-岗位关系错误 ============
    #[error("员工岗位关系不存在")]
    EmployeePositionRelNotFound,

    #[error("员工岗位关系已存在")]
    EmployeePositionRelExists,

    #[error("员工不能移出最后一个岗位")]
    EmployeeCannotRemoveLastPosition,

    // ============ 数据库错误 ============
    #[error("数据库连接失败：{0}")]
    DatabaseConnectError(String),

    #[error("数据库查询失败：{0}")]
    DatabaseQueryError(String),

    #[error("数据库事务失败：{0}")]
    DatabaseTransactionError(String),

    #[error("处理数据库错误：{0}")]
    DatabaseError(String),

    // ============ 并发控制错误 ============
    #[error("并发冲突：数据已被修改")]
    ConcurrentConflict,

    #[error("数据一致性检查失败：{0}")]
    DataConsistencyError(String),

    // ============ 参数验证错误 ============
    #[error("参数为空")]
    ParamNull,

    #[error("参数类型错误：{0}")]
    ParamTypeError(String),

    #[error("参数值超出范围：{0}")]
    ParamValueOutOfRange(String),

    #[error("参数格式错误：{0}")]
    ParamFormatError(String),

    #[error("必填参数缺失：{0}")]
    ParamRequired(String),

    #[error("分页参数错误：{0}")]
    PaginationError(String),

    #[error("排序参数错误：{0}")]
    OrderError(String),

    #[error("时间参数错误：{0}")]
    DateError(String),

    // ============ 业务逻辑错误 ============
    #[error("业务规则冲突：{0}")]
    BusinessConflict(String),

    #[error("操作状态非法：{0}")]
    BusinessStateError(String),

    #[error("批量操作部分失败：{0}")]
    BusinessPartialFailure(String),

    #[error("导出数据为空")]
    NoDataToExport,

    #[error("导入数据格式错误：{0}")]
    ImportFormatError(String),

    #[error("导入数据验证失败：{0}")]
    ImportValidationFailed(String),

    // ============ 权限与认证错误 ============
    #[error("无权限执行此操作")]
    PermissionDenied,

    #[error("认证失败")]
    Unauthorized,

    #[error("令牌过期")]
    TokenExpired,

    #[error("令牌无效")]
    TokenInvalid,

    // ============ 系统错误 ============
    #[error("系统内部错误：{0}")]
    InternalServerError(String),

    #[error("服务暂时不可用")]
    ServiceUnavailable,

    #[error("操作超时")]
    OperationTimeout,

    #[error("资源冲突：{0}")]
    ResourceConflict(String),

    #[error("缓存错误：{0}")]
    CacheError(String),

    // ============ 通讯录模块错误 ============
    #[error("搜索引擎错误：{0}")]
    ContactsSearchFailed(String),

    #[error("部门成员数超过上限（{0}），请按子部门分别查看")]
    ContactsMemberCountExceeded(u32),

    #[error("该员工在通讯录中不可见")]
    ContactsEmployeeNotVisible,

    #[error("该部门在通讯录中不可见")]
    ContactsDepartmentNotVisible,
}

impl OrganizationError {
    /// 获取错误码（根据错误类型映射到对应的错误码）
    pub fn code(&self) -> i32 {
        use error_code::*;
        match self {
            // 组织模块
            OrganizationError::OrganizationNotFound => ORGANIZATION_NOT_FOUND,
            OrganizationError::OrganizationExists => ORGANIZATION_EXISTS,
            OrganizationError::OrganizationCodeDuplicate(_) => ORGANIZATION_CODE_DUP,
            OrganizationError::OrganizationNameDuplicate(_) => ORGANIZATION_NAME_DUP,
            OrganizationError::OrganizationHasChildren => ORGANIZATION_HAS_CHILDREN,
            OrganizationError::OrganizationHasEmployees => ORGANIZATION_HAS_DEPT,
            OrganizationError::OrganizationCodeInvalid(_) => ORGANIZATION_CODE_INVALID,
            OrganizationError::OrganizationNameEmpty => ORGANIZATION_NAME_EMPTY,
            OrganizationError::OrganizationUnauthorized => ORGANIZATION_UNAUTHORIZED,
            OrganizationError::OrganizationStatusInvalid => ORGANIZATION_STATUS_INVALID,

            // 部门模块
            OrganizationError::DepartmentNotFound => DEPARTMENT_NOT_FOUND,
            OrganizationError::DepartmentExists(_) => DEPARTMENT_EXISTS,
            OrganizationError::DepartmentCodeDuplicate(_) => DEPARTMENT_CODE_DUP,
            OrganizationError::DepartmentHasChildren => DEPARTMENT_HAS_CHILDREN,
            OrganizationError::DepartmentHasEmployees => DEPARTMENT_HAS_EMPLOYEES,
            OrganizationError::DepartmentCodeInvalid(_) => DEPARTMENT_CODE_INVALID,
            OrganizationError::DepartmentNameEmpty => DEPARTMENT_NAME_EMPTY,
            OrganizationError::DepartmentParentNotFound => DEPARTMENT_PARENT_NOT_FOUND,
            OrganizationError::DepartmentParentSelf => DEPARTMENT_PARENT_SELF,
            OrganizationError::DepartmentUnauthorized => DEPARTMENT_UNAUTHORIZED,
            OrganizationError::DepartmentLevelTooDeep => DEPARTMENT_LEVEL_TOO_DEEP,
            OrganizationError::DepartmentStatusInvalid => DEPARTMENT_STATUS_INVALID,

            // 岗位模块
            OrganizationError::PositionNotFound => POSITION_NOT_FOUND,
            OrganizationError::PositionExists(_) => POSITION_EXISTS,
            OrganizationError::PositionCodeDuplicate(_) => POSITION_CODE_DUP,
            OrganizationError::PositionHasEmployees => POSITION_HAS_EMPLOYEES,
            OrganizationError::PositionNameEmpty => POSITION_NAME_EMPTY,
            OrganizationError::PositionCodeInvalid(_) => POSITION_CODE_INVALID,
            OrganizationError::PositionLevelInvalid => POSITION_LEVEL_INVALID,
            OrganizationError::PositionUnauthorized => POSITION_UNAUTHORIZED,
            OrganizationError::PositionStatusInvalid => POSITION_STATUS_INVALID,

            // 员工模块
            OrganizationError::EmployeeNotFound => EMPLOYEE_NOT_FOUND,
            OrganizationError::EmployeeExists => EMPLOYEE_EXISTS,
            OrganizationError::EmployeeNoExists(_) => EMPLOYEE_NO_EXISTS,
            OrganizationError::UserAlreadyEmployee => USER_ALREADY_EMPLOYEE,
            OrganizationError::EmployeeNoEmpty => EMPLOYEE_NO_EMPTY,
            OrganizationError::EmployeeNameEmpty => EMPLOYEE_NAME_EMPTY,
            OrganizationError::EmployeeNoInvalid(_) => EMPLOYEE_NO_INVALID,
            OrganizationError::EmployeeEmailInvalid(_) => EMPLOYEE_EMAIL_INVALID,
            OrganizationError::EmployeePhoneInvalid(_) => EMPLOYEE_PHONE_INVALID,
            OrganizationError::EmployeeIdCardInvalid => EMPLOYEE_ID_CARD_INVALID,
            OrganizationError::EmployeeUnauthorized => EMPLOYEE_UNAUTHORIZED,
            OrganizationError::EmployeeStatusInvalid => EMPLOYEE_STATUS_INVALID,
            OrganizationError::EmployeeNoDepartment => EMPLOYEE_NO_DEPT,

            // 员工-部门关系
            OrganizationError::EmployeeDepartmentRelNotFound => EMPLOYEE_DEPT_REL_NOT_FOUND,
            OrganizationError::EmployeeDepartmentRelExists => EMPLOYEE_DEPT_REL_EXISTS,
            OrganizationError::EmployeeCannotRemoveLastDepartment => EMPLOYEE_DEPT_LAST,
            OrganizationError::EmployeeNoDepartmentPosition => EMPLOYEE_DEPT_NO_POSITION,

            // 员工-岗位关系
            OrganizationError::EmployeePositionRelNotFound => EMPLOYEE_POSITION_REL_NOT_FOUND,
            OrganizationError::EmployeePositionRelExists => EMPLOYEE_POSITION_REL_EXISTS,
            OrganizationError::EmployeeCannotRemoveLastPosition => EMPLOYEE_POSITION_LAST,

            // 数据库错误
            OrganizationError::DatabaseConnectError(_) => DATABASE_CONNECT_ERROR,
            OrganizationError::DatabaseQueryError(_) => DATABASE_QUERY_ERROR,
            OrganizationError::DatabaseTransactionError(_) => DATABASE_TRANSACTION_ERROR,
            OrganizationError::DatabaseError(_) => DATABASE_QUERY_ERROR,

            // 并发控制
            OrganizationError::ConcurrentConflict => CONCURRENT_CONFLICT,
            OrganizationError::DataConsistencyError(_) => DATA_CONSISTENCY_ERROR,

            // 参数验证
            OrganizationError::ParamNull => PARAM_NULL,
            OrganizationError::ParamTypeError(_) => PARAM_TYPE_ERROR,
            OrganizationError::ParamValueOutOfRange(_) => PARAM_VALUE_OUT_OF_RANGE,
            OrganizationError::ParamFormatError(_) => PARAM_FORMAT_ERROR,
            OrganizationError::ParamRequired(_) => PARAM_REQUIRED,
            OrganizationError::PaginationError(_) => PARAM_PAGINATION_ERROR,
            OrganizationError::OrderError(_) => PARAM_ORDER_ERROR,
            OrganizationError::DateError(_) => PARAM_DATE_ERROR,

            // 业务逻辑
            OrganizationError::BusinessConflict(_) => BUSINESS_CONFLICT,
            OrganizationError::BusinessStateError(_) => BUSINESS_STATE_ERROR,
            OrganizationError::BusinessPartialFailure(_) => BUSINESS_PARTIAL_FAILURE,
            OrganizationError::NoDataToExport => BUSINESS_NO_DATA_TO_EXPORT,
            OrganizationError::ImportFormatError(_) => BUSINESS_IMPORT_FORMAT_ERROR,
            OrganizationError::ImportValidationFailed(_) => BUSINESS_IMPORT_VALIDATION_FAILED,

            // 权限与认证
            OrganizationError::PermissionDenied => PERMISSION_DENIED,
            OrganizationError::Unauthorized => UNAUTHORIZED,
            OrganizationError::TokenExpired => TOKEN_EXPIRED,
            OrganizationError::TokenInvalid => TOKEN_INVALID,

            // 系统错误
            OrganizationError::InternalServerError(_) => INTERNAL_SERVER_ERROR,
            OrganizationError::ServiceUnavailable => SERVICE_UNAVAILABLE,
            OrganizationError::OperationTimeout => OPERATION_TIMEOUT,
            OrganizationError::ResourceConflict(_) => RESOURCE_CONFLICT,
            OrganizationError::CacheError(_) => CACHE_ERROR,

            // 通讯录模块
            OrganizationError::ContactsSearchFailed(_) => CONTACTS_SEARCH_FAILED,
            OrganizationError::ContactsMemberCountExceeded(_) => CONTACTS_MEMBER_COUNT_EXCEEDED,
            OrganizationError::ContactsEmployeeNotVisible => CONTACTS_EMPLOYEE_NOT_VISIBLE,
            OrganizationError::ContactsDepartmentNotVisible => CONTACTS_DEPARTMENT_NOT_VISIBLE,
        }
    }

    /// 获取HTTP状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 404 Not Found
            OrganizationError::OrganizationNotFound
            | OrganizationError::DepartmentNotFound
            | OrganizationError::PositionNotFound
            | OrganizationError::EmployeeNotFound
            | OrganizationError::EmployeeDepartmentRelNotFound
            | OrganizationError::EmployeePositionRelNotFound
            | OrganizationError::ContactsEmployeeNotVisible
            | OrganizationError::ContactsDepartmentNotVisible => StatusCode::NOT_FOUND,

            // 400 Bad Request - 参数错误
            OrganizationError::ParamNull
            | OrganizationError::ParamTypeError(_)
            | OrganizationError::ParamValueOutOfRange(_)
            | OrganizationError::ParamFormatError(_)
            | OrganizationError::ParamRequired(_)
            | OrganizationError::PaginationError(_)
            | OrganizationError::OrderError(_)
            | OrganizationError::DateError(_)
            | OrganizationError::ContactsMemberCountExceeded(_)
            => StatusCode::BAD_REQUEST,

            // 409 Conflict - 业务冲突
            OrganizationError::OrganizationExists
            | OrganizationError::OrganizationCodeDuplicate(_)
            | OrganizationError::OrganizationNameDuplicate(_)
            | OrganizationError::DepartmentExists(_)
            | OrganizationError::DepartmentCodeDuplicate(_)
            | OrganizationError::PositionExists(_)
            | OrganizationError::PositionCodeDuplicate(_)
            | OrganizationError::EmployeeExists
            | OrganizationError::EmployeeNoExists(_)
            | OrganizationError::DepartmentHasChildren
            | OrganizationError::DepartmentHasEmployees
            | OrganizationError::PositionHasEmployees
            | OrganizationError::ConcurrentConflict
            | OrganizationError::EmployeeDepartmentRelExists
            | OrganizationError::EmployeePositionRelExists
            => StatusCode::CONFLICT,

            // 401 Unauthorized
            OrganizationError::Unauthorized
            | OrganizationError::TokenExpired
            | OrganizationError::TokenInvalid
            => StatusCode::UNAUTHORIZED,

            // 403 Forbidden
            OrganizationError::PermissionDenied
            | OrganizationError::OrganizationUnauthorized
            | OrganizationError::DepartmentUnauthorized
            | OrganizationError::PositionUnauthorized
            | OrganizationError::EmployeeUnauthorized
            => StatusCode::FORBIDDEN,

            // 503 Service Unavailable
            OrganizationError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,

            // 408 ReqThread Timeout
            OrganizationError::OperationTimeout => StatusCode::REQUEST_TIMEOUT,

            // 500 Internal Server Error - 其他所有错误
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// 从数据库错误转换
impl From<sqlx::Error> for OrganizationError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => OrganizationError::DatabaseQueryError("查询无结果".to_string()),
            sqlx::Error::PoolClosed => OrganizationError::DatabaseConnectError("连接池已关闭".to_string()),
            sqlx::Error::PoolTimedOut => OrganizationError::DatabaseConnectError("获取连接池超时".to_string()),
            _ => OrganizationError::DatabaseQueryError(err.to_string()),
        }
    }
}

/// 实现 IntoResponse 以便在 Handler 中直接返回错误
impl IntoResponse for OrganizationError {
    fn into_response(self) -> Response {
        let code = self.code();
        let message = self.to_string();
        let status = self.status_code();

        tracing::warn!(
            error_code = code,
            error_message = %message,
            status = %status,
            "API错误响应"
        );

        (status, Json(R::<()>::fail_with_code(code, message))).into_response()
    }
}

/// 统一的 Result 类型别名 - 所有 Repository 和 Service 层使用
pub type Result<T> = std::result::Result<T, OrganizationError>;
