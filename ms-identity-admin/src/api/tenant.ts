import { AxiosResponse } from 'axios';
import request from '@/utils/request';
import { IApiResult, ITenantInfo, IPageResult, IApplicationInfo } from '@/types/base';

export interface ICreateTenantParam {
  name: string;
  contact_name: string;
  contact_mobile?: string;
  website?: string;
  package_id: number;
  expire_time: string; // ISO 8601 format
  account_count: number;
}

export interface IUpdateTenantParam {
  name?: string;
  contact_name?: string;
  contact_mobile?: string;
  website?: string;
  status?: number;
}

export interface ITenantApplicationInfo {
  id?: number;
  tenant_id: number;
  application_id: number;
  create_time?: string;
}

export interface IAddApplicationToTenantParam {
  application_id: number;
}

class TenantApi {
  // 创建租户
  createTenant(info: ICreateTenantParam): Promise<AxiosResponse<IApiResult<{ tenant_id: number }>>> {
    return request.requestJSON({
      method: 'post',
      url: '/api/v1/identity/tenants',
      data: info
    });
  }

  // 获取租户信息
  getTenant(id: number): Promise<AxiosResponse<IApiResult<ITenantInfo>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/tenants/${id}`
    });
  }

  // 更新租户
  updateTenant(id: number, info: IUpdateTenantParam): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'put',
      url: `/api/v1/identity/tenants/${id}`,
      data: info
    });
  }

  // 删除租户
  deleteTenant(id: number): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'delete',
      url: `/api/v1/identity/tenants/${id}`
    });
  }

  // 获取租户列表（分页）
  getTenantList(params: {
    page_size?: number;
    cursor?: number;
    search_key?: string;
  }): Promise<AxiosResponse<IApiResult<IPageResult<ITenantInfo>>>> {
    // 过滤掉 undefined 值
    const cleanParams: Record<string, any> = {};
    if (params.page_size !== undefined) cleanParams.page_size = params.page_size;
    if (params.cursor !== undefined) cleanParams.cursor = params.cursor;
    if (params.search_key !== undefined) cleanParams.search_key = params.search_key;
    
    return request.requestJSON({
      method: 'get',
      url: '/api/v1/identity/tenants',
      params: cleanParams
    });
  }

  // 获取租户的应用列表（返回应用详细信息，不是关系表数据）
  getTenantApplications(tenantId: number): Promise<AxiosResponse<IApiResult<IApplicationInfo[]>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/tenants/${tenantId}/applications`
    });
  }

  // 添加应用到租户
  addApplicationToTenant(tenantId: number, applicationId: number): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'post',
      url: `/api/v1/identity/tenants/${tenantId}/applications`,
      data: { application_id: applicationId }
    });
  }

  // 从租户移除应用
  removeApplicationFromTenant(tenantId: number, applicationId: number): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'delete',
      url: `/api/v1/identity/tenants/${tenantId}/applications`,
      data: { application_id: applicationId }
    });
  }

  // 获取租户总数
  getTenantCount(): Promise<AxiosResponse<IApiResult<number>>> {
    return request.requestJSON({
      method: 'get',
      url: '/api/v1/identity/tenants/count'
    });
  }
}

export const tenantApi = new TenantApi();

