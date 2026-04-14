// API 响应基础类型
export interface IApiResult<T = any> {
  success: boolean;
  code: number | string;
  msg: string;
  data: T;
  path?: string;
  version?: string;
  base_version?: string;
  timestamp?: number;
}

// 分页结果（游标分页）
export interface IPageResult<T> {
  list: T[];
  total: number;
  cursor?: number;
  has_next: boolean;
}

// 分页参数
export interface IPageParam {
  page?: number;
  page_size?: number;
  cursor?: number;
}

// 用户信息
export interface IUserInfo {
  id?: number;
  username?: string;
  email?: string;
  mobile?: string;
  nick_name?: string;
  real_name?: string;
  state?: number;
  user_type?: number;
  create_time?: string;
  update_time?: string;
}

// 租户信息
export interface ITenantInfo {
  id?: number;
  name: string;
  contact_name: string;
  contact_mobile?: string;
  contact_user_id?: number;
  status?: number;
  package_id?: number;
  expire_time?: string;
  account_count?: number;
  website?: string;
  create_time?: string;
  update_time?: string;
}

// 角色信息
export interface IRoleInfo {
  id?: number;
  code: string;
  name: string;
  tenant_id: number;
  remarks?: string;
  state?: boolean;
  create_time?: string;
  update_time?: string;
}

// 资源信息
export interface IResourceInfo {
  id?: number;
  application_id: number;
  code: string;
  name: string;
  parent_id: number;
  resource_type?: string; // 资源类型：20-菜单 40-按钮 50-字段 60-数据
  path?: string;
  component?: string;
  icon?: string;
  describe_?: string; // 描述
  state?: boolean; // 状态：0-禁用 1-启用
  sort_value?: number; // 排序值，默认升序
  create_time?: string;
  update_time?: string;
}

// 菜单下按类型分类的子资源
export interface IMenuResourcesByType {
  menus: IResourceInfo[];
  buttons: IResourceInfo[];
  fields: IResourceInfo[];
  data: IResourceInfo[];
}

// 应用信息
export interface IApplicationInfo {
  id?: number;
  name?: string;
  type?: string; // 应用类型
  app_key?: string; // 应用标识
  version?: string; // 版本
  redirect?: string; // 重定向地址
  url?: string; // 应用地址
  introduce?: string; // 简介
  remark?: string; // 备注
  is_general?: boolean; // 是否公共应用
  is_visible?: boolean; // 是否可见
  sort_value?: number; // 排序
  create_time?: string; // 创建时间
  update_time?: string; // 更新时间
}

// 套餐信息
export interface IPlanInfo {
  id?: number;
  name: string;
  type: string;
  price?: string | number;
  billing_cycle: string;
  description?: string;
  is_active?: boolean;
  sort_order?: number;
}

