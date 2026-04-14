import { AxiosResponse } from 'axios';
import request from '@/utils/request';
import { IApiResult, IPageResult, IRoleInfo } from '@/types/base';

export interface ICreateRoleParam {
  code: string;
  name: string;
  tenant_id: number;
  remarks?: string;
  state?: boolean;
}

export interface IUpdateRoleParam {
  name?: string;
  remarks?: string;
  state?: boolean;
}

export interface ICreateRoleResponse {
  role_id: number;
}

export const roleApi = {
  // 获取角色列表（分页）
  getRoleList(params: {
    page_size?: number;
    cursor?: number;
    tenant_id?: number;
  }): Promise<AxiosResponse<IApiResult<IPageResult<IRoleInfo>>>> {
    const cleanParams: Record<string, any> = {};
    if (params.page_size !== undefined) cleanParams.page_size = params.page_size;
    if (params.cursor !== undefined) cleanParams.cursor = params.cursor;
    if (params.tenant_id !== undefined) cleanParams.tenant_id = params.tenant_id;
    
    return request.requestJSON({
      method: 'get',
      url: '/api/v1/identity/roles',
      params: cleanParams
    });
  },

  // 获取角色详情
  getRole(id: number): Promise<AxiosResponse<IApiResult<IRoleInfo>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/roles/${id}`
    });
  },

  // 创建角色
  createRole(params: ICreateRoleParam): Promise<AxiosResponse<IApiResult<ICreateRoleResponse>>> {
    return request.requestJSON({
      method: 'post',
      url: '/api/v1/identity/roles',
      data: params
    });
  },

  // 更新角色
  updateRole(id: number, params: IUpdateRoleParam): Promise<AxiosResponse<IApiResult<void>>> {
    return request.requestJSON({
      method: 'put',
      url: `/api/v1/identity/roles/${id}`,
      data: params
    });
  },

  // 删除角色
  deleteRole(id: number): Promise<AxiosResponse<IApiResult<void>>> {
    return request.requestJSON({
      method: 'delete',
      url: `/api/v1/identity/roles/${id}`
    });
  },

  // 获取租户的角色列表（不分页）
  getTenantRoles(tenantId: number): Promise<AxiosResponse<IApiResult<IRoleInfo[]>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/roles/tenant/${tenantId}`
    });
  },

  // 获取角色的资源列表
  getRoleResources(roleId: number): Promise<AxiosResponse<IApiResult<any[]>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/roles/${roleId}/resources`
    });
  },

  // 分配资源到角色
  assignResourceToRole(roleId: number, resourceId: number): Promise<AxiosResponse<IApiResult<void>>> {
    return request.requestJSON({
      method: 'post',
      url: `/api/v1/identity/roles/${roleId}/resources`,
      data: { resource_id: resourceId }
    });
  },

  // 从角色移除资源
  removeResourceFromRole(roleId: number, resourceId: number): Promise<AxiosResponse<IApiResult<void>>> {
    return request.requestJSON({
      method: 'delete',
      url: `/api/v1/identity/roles/${roleId}/resources`,
      data: { resource_id: resourceId }
    });
  }
};

