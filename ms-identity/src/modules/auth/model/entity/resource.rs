/// Resource
/// 
/// 表名: `resource`
/// 主键: `id`
/// 逻辑删除字段: `is_del`
/// 字段数: 30

#[derive(Debug, Default, sqlx::FromRow, serde::Serialize, serde::Deserialize, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "resource", pk = "id", soft_delete = "is_del", table_comment = "资源")]
pub struct Resource {
    /// 主键 | id (bigint) | 非空
    #[column(primary_key, auto_increment, comment = "ID")]
    pub id: Option<i64>,
    /// application_id (bigint) | 非空
    #[column(not_null, comment = "应用ID;#def_application")]
    pub application_id: i64,
    /// code (varchar(255)) | 非空
    #[column(not_null, length = 255, unique, index, comment = "编码;唯一编码，用于区分资源")]
    pub code: String,
    /// name (varchar(255)) | 非空
    #[column(not_null, length = 255, comment = "名称")]
    pub name: String,
    /// resource_type (char(2)) | 非空
    /// 默认值: 20
    #[column(not_null, default = "20", length = 2, comment = "类型;[20-菜单 40-按钮 50-字段 60-数据]@Echo(api = EchoApi.DICTIONARY_ITEM_FEIGN_CLASS,dictType = EchoDictType.System.RESOURCE_TYPE)菜单即左侧显示的菜单视图即隐藏的菜单(需要配置在路由中)和页面上点击后需要通过路由打开的页面功能即页面上的非视图的按钮字段即列表页或编辑页的字段接口即后台的访问接口")]
    pub resource_type: Option<String>,
    /// parent_id (bigint) | 非空
    #[column(not_null, comment = "父级ID")]
    pub parent_id: i64,
    /// open_with (char(2)) | 可空
    /// 默认值: 01
    #[column(default = "01", length = 2, comment = "打开方式;[01-组件 02-内链 03-外链]
@Echo(api = EchoApi.DICTIONARY_ITEM_FEIGN_CLASS, dictType = EchoDictType.System.RESOURCE_OPEN_WITH)")]
    pub open_with: Option<String>,
    /// describe_ (varchar(255)) | 可空
    /// 默认值: 
    #[column(length = 255, comment = "描述;resource_type=接口时表示接口说明")]
    pub describe_: Option<String>,
    /// path (varchar(255)) | 可空
    /// 默认值: 
    #[column(length = 255, comment = "地址栏路径;用于resource_type=菜单和视图和接口.resource_type=菜单和视图，表示地址栏地址, http开头表示外链, is_frame_src 为true表示在框架类打开.resource_type=接口，表示后端接口请求地址.")]
    pub path: Option<String>,
    /// component (varchar(255)) | 可空
    /// 默认值: 
    #[column(length = 255, comment = "页面路径;用于resource_type=菜单和视图. 前端页面在src/views目录下的相对地址.")]
    pub component: Option<String>,
    /// redirect (varchar(255)) | 可空
    /// 默认值: 
    #[column(length = 255, comment = "重定向;用于resource_type=菜单和视图")]
    pub redirect: Option<String>,
    /// icon (varchar(255)) | 可空
    /// 默认值: 
    #[column(length = 255, comment = "图标")]
    pub icon: Option<String>,
    /// is_hidden (bit(1)) | 可空
    /// 默认值: b'0'
    #[column(default = "b\'0\'", length = 1, comment = "是否隐藏菜单;
resource_type=20时生效")]
    pub is_hidden: Option<bool>,
    /// is_general (bit(1)) | 可空
    /// 默认值: b'0'
    #[column(default = "b\'0\'", length = 1, comment = "是否公共资源;1-无需分配所有人就可以访问的")]
    pub is_general: Option<bool>,
    /// state (bit(1)) | 非空
    /// 默认值: b'1'
    #[column(not_null, default = "b\'1\'", length = 1, comment = "状态;[0-禁用 1-启用]")]
    pub state: Option<bool>,
    /// sort_value (int) | 可空
    /// 默认值: 1
    #[column(default = "1", comment = "排序;默认升序")]
    pub sort_value: Option<i32>,
    /// sub_group (varchar(255)) | 可空
    /// 默认值: 
    #[column(length = 255, comment = "分组")]
    pub sub_group: Option<String>,
    /// field_is_secret (bit(1)) | 可空
    /// 默认值: b'0'
    #[column(default = "b\'0\'", length = 1, comment = "是否脱敏;显示时是否需要脱敏实现 (用于resource_type=字段)")]
    pub field_is_secret: Option<bool>,
    /// field_is_edit (bit(1)) | 可空
    /// 默认值: b'1'
    #[column(default = "b\'1\'", length = 1, comment = "是否可以编辑;是否可以编辑(用于resource_type=字段)")]
    pub field_is_edit: Option<bool>,
    /// data_scope (char(2)) | 可空
    #[column(length = 2, comment = "数据范围;[01-全部 02-本单位及子级 03-本单位 04-本部门及子级 05-本部门 06-个人 07-自定义]")]
    pub data_scope: Option<String>,
    /// custom_class (varchar(255)) | 可空
    #[column(length = 255, comment = "实现类;自定义实现类全类名")]
    pub custom_class: Option<String>,
    /// is_def (bit(1)) | 可空
    /// 默认值: b'0'
    #[column(default = "b\'0\'", length = 1, comment = "是否默认")]
    pub is_def: Option<bool>,
    /// tree_path (varchar(512)) | 可空
    /// 默认值: /
    #[column(default = "/", length = 512, comment = "树路径")]
    pub tree_path: Option<String>,
    /// tree_grade (int) | 可空
    /// 默认值: 0
    #[column(default = "0", comment = "树层级")]
    pub tree_grade: Option<i32>,
    /// meta_json (varchar(512)) | 可空
    /// 默认值: {}
    #[column(default = "{}", length = 512, comment = "元数据;菜单视图的元数据")]
    pub meta_json: Option<String>,
    /// create_by (bigint) | 可空
    #[column(comment = "创建人id")]
    pub create_by: Option<i64>,
    /// create_time (timestamp(3)) | 可空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "创建时间")]
    pub create_time: Option<chrono::DateTime<chrono::Utc>>,
    /// update_by (bigint) | 可空
    #[column(comment = "更新人id")]
    pub update_by: Option<i64>,
    /// update_time (timestamp(3)) | 可空
    /// 默认值: CURRENT_TIMESTAMP(3)
    #[column(default = "CURRENT_TIMESTAMP(3)", length = 3, comment = "更新时间")]
    pub update_time: Option<chrono::DateTime<chrono::Utc>>,
    /// is_del (tinyint(1)) | 非空
    /// 默认值: 0
    #[column(not_null, default = "0", length = 1, soft_delete, comment = "")]
    pub is_del: Option<bool>,
}
