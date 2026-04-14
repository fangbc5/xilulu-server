import { AxiosResponse } from 'axios';
import request from '@/utils/request';
import { IApiResult, IUserInfo, IPageResult } from '@/types/base';

export interface IUserTenantInfo {
  id?: number;
  user_id: number;
  tenant_id: number;
  is_owner?: number;
  status?: number;
  join_time?: string;
}

export interface ILoginParam {
  username?: string;
  password?: string;
  mobile?: string;
  email?: string;
  code?: string;
  captcha_id?: string;
  captcha?: string;
}

export interface IRegisterParam {
  username?: string;
  password?: string;
  mobile?: string;
  email?: string;
  code?: string;
  nick_name?: string;
}

export interface ICreateUserParam {
  username: string;
  password: string;
  email?: string;
  mobile?: string;
  nick_name?: string;
}

export interface IUpdateUserParam {
  username?: string;
  email?: string;
  mobile?: string;
  nick_name?: string;
}

export interface IChangePasswordParam {
  old_password: string;
  new_password: string;
}

class UserApi {
  // 登录
  login(info: ILoginParam): Promise<AxiosResponse<IApiResult<{ access_token: string; refresh_token: string; user_info: any; tenant_list?: any[] }>>> {
    return request.requestJSON({
      method: 'post',
      url: '/api/v1/auth/login',
      data: info
    });
  }

  // 注册
  register(info: IRegisterParam): Promise<AxiosResponse<IApiResult<{ user_id: number }>>> {
    return request.requestJSON({
      method: 'post',
      url: '/api/v1/auth/register',
      data: info
    });
  }

  // 发送验证码
  sendCode(mobile?: string, email?: string): Promise<AxiosResponse<IApiResult<{ message: string }>>> {
    return request.requestJSON({
      method: 'post',
      url: '/api/v1/auth/send-code',
      data: { mobile, email }
    });
  }

  // 获取图片验证码
  getCaptcha(): Promise<AxiosResponse<IApiResult<{ captcha_id: string; image_base64: string }>>> {
    return request.requestJSON({
      method: 'get',
      url: '/api/v1/auth/captcha'
    });
  }

  // 刷新 token
  refreshToken(refresh_token: string): Promise<AxiosResponse<IApiResult<{ access_token: string; refresh_token: string }>>> {
    return request.requestJSON({
      method: 'post',
      url: '/api/v1/auth/refresh-token',
      data: { refresh_token }
    });
  }

  // 登出
  logout(): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'post',
      url: '/api/v1/auth/logout'
    });
  }

  // 创建用户
  createUser(info: ICreateUserParam): Promise<AxiosResponse<IApiResult<{ user_id: number }>>> {
    return request.requestJSON({
      method: 'post',
      url: '/api/v1/identity/users',
      data: info
    });
  }

  // 获取用户信息
  getUser(id: number): Promise<AxiosResponse<IApiResult<IUserInfo>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/users/${id}`
    });
  }

  // 更新用户
  updateUser(id: number, info: IUpdateUserParam): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'put',
      url: `/api/v1/identity/users/${id}`,
      data: info
    });
  }

  // 删除用户
  deleteUser(id: number): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'delete',
      url: `/api/v1/identity/users/${id}`
    });
  }

  // 修改密码
  changePassword(id: number, info: IChangePasswordParam): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'put',
      url: `/api/v1/identity/users/${id}/password`,
      data: info
    });
  }

  // 重置密码
  resetPassword(id: number, new_password: string): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'put',
      url: `/api/v1/identity/users/${id}/password/reset`,
      data: { new_password }
    });
  }

  // 获取用户租户列表
  getUserTenants(id: number): Promise<AxiosResponse<IApiResult<IUserTenantInfo[]>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/users/${id}/tenants`
    });
  }

  // 添加用户到租户
  addUserToTenant(userId: number, tenantId: number): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'post',
      url: `/api/v1/identity/users/${userId}/tenants`,
      data: { tenant_id: tenantId }
    });
  }

  // 从租户移除用户
  removeUserFromTenant(userId: number, tenantId: number): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'delete',
      url: `/api/v1/users/${userId}/tenants`,
      data: { tenant_id: tenantId }
    });
  }

  // 设置默认租户
  setDefaultTenant(userId: number, tenantId: number): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'put',
      url: `/api/v1/identity/users/${userId}/tenants/default`,
      data: { tenant_id: tenantId }
    });
  }

  // 获取用户列表（分页）
  getUserList(params: {
    page_size?: number;
    cursor?: number;
    search_key?: string;
    tenant_id?: number;
  }): Promise<AxiosResponse<IApiResult<IPageResult<IUserInfo>>>> {
    // 过滤掉 undefined 值
    const cleanParams: Record<string, any> = {};
    if (params.page_size !== undefined) cleanParams.page_size = params.page_size;
    if (params.cursor !== undefined) cleanParams.cursor = params.cursor;
    if (params.search_key !== undefined) cleanParams.search_key = params.search_key;
    if (params.tenant_id !== undefined) cleanParams.tenant_id = params.tenant_id;
    
    return request.requestJSON({
      method: 'get',
      url: '/api/v1/identity/users',
      params: cleanParams
    });
  }

  // 获取用户在租户下的角色列表
  getUserRoles(userId: number, tenantId: number): Promise<AxiosResponse<IApiResult<IUserRoleInfo[]>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/users/${userId}/tenants/${tenantId}/roles`
    });
  }

  // 获取用户总数
  getUserCount(): Promise<AxiosResponse<IApiResult<number>>> {
    return request.requestJSON({
      method: 'get',
      url: '/api/v1/identity/users/count'
    });
  }

  // 获取活跃用户数
  getActiveUserCount(days?: number): Promise<AxiosResponse<IApiResult<number>>> {
    const params: Record<string, any> = {};
    if (days !== undefined) {
      params.days = days;
    }
    return request.requestJSON({
      method: 'get',
      url: '/api/v1/identity/users/count/active',
      params
    });
  }

  // 为用户分配角色
  assignRoleToUser(userId: number, tenantId: number, roleId: number): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'post',
      url: `/api/v1/identity/users/${userId}/tenants/${tenantId}/roles`,
      data: { role_id: roleId }
    });
  }

  // 批量为用户分配角色
  batchAssignRolesToUser(userId: number, tenantId: number, roleIds: number[]): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'post',
      url: `/api/v1/identity/users/${userId}/tenants/${tenantId}/roles/batch`,
      data: { role_ids: roleIds }
    });
  }

  // 移除用户角色
  removeRoleFromUser(userId: number, tenantId: number, roleId: number): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'delete',
      url: `/api/v1/identity/users/${userId}/tenants/${tenantId}/roles/${roleId}`
    });
  }
}

export interface IUserRoleInfo {
  id?: number;
  user_id: number;
  role_id: number;
  role_code: string;
  tenant_id: number;
  created_at?: string;
}

export const userApi = new UserApi();

