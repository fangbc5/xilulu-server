pub const SUCCESS_CODE: i32 = 200;
pub const SUCCESS_MESSAGE: &str = "ok";
pub const DEFAULT_ERROR_MESSAGE: &str = "系统繁忙，请稍候再试";
pub const HYSTRIX_ERROR_MESSAGE: &str = "请求超时，请稍候再试";
pub const FAIL_CODE: i32 = -1;
pub const TIMEOUT_CODE: i32 = -2;

///统一参数验证异常
pub const VALID_EX_CODE: i32 = -9;
pub const OPERATION_EX_CODE: i32 = -10;

///必须表字段
pub const ID_FIELD: &str = "id";
pub const CREATE_TIME: &str = "createTime";
pub const CREATE_TIME_FIELD: &str = "create_time";
pub const CREATE_BY: &str = "createBy";
pub const CREATE_BY_FIELD: &str = "create_by";
pub const TENANT_ID: &str = "tenant_id";
pub const CREATE_ORG_ID_FIELD: &str = "create_org_id";
pub const DELETE_FIELD: &str = "is_del";
/// 更新字段
pub const UPDATE_TIME: &str = "updateTime";
pub const UPDATE_BY: &str = "updateBy";
pub const UPDATE_TIME_FIELD: &str = "update_time";
pub const UPDATE_BY_FIELD: &str = "update_by";
/// 树表字段
pub const LABEL: &str = "label";
pub const PARENT_ID: &str = "parentId";
pub const SORT_VALUE: &str = "sortValue";
pub const PARENT_ID_FIELD: &str = "parent_id";
pub const SORT_VALUE_FIELD: &str = "sort_value";

/// 请求头中的key
pub const HEADER_KEY_UID: &str = "uid";
pub const HEADER_KEY_TENANT_ID: &str = "tenant_id";

