import { AxiosResponse } from 'axios';
import request from '@/utils/request';
import { IApiResult, IResourceInfo, IPageResult, IMenuResourcesByType } from '@/types/base';

export interface ICreateResourceParam {
  application_id: number;
  code: string;
  name: string;
  parent_id: number;
  resource_type?: string; // 资源类型：20-菜单 40-按钮 50-字段 60-数据
  path?: string;
  describe_?: string; // 描述
  state?: boolean; // 状态
}

export interface IUpdateResourceParam {
  name?: string;
  path?: string;
  describe_?: string; // 描述
}

class ResourceApi {
  // 获取资源列表（分页）
  getResourceList(params: {
    page_size?: number;
    cursor?: number;
    application_id?: number;
    tenant_id?: number;
    search_key?: string;
  }): Promise<AxiosResponse<IApiResult<IPageResult<IResourceInfo>>>> {
    // 过滤掉 undefined 值
    const cleanParams: Record<string, any> = {};
    if (params.page_size !== undefined) cleanParams.page_size = params.page_size;
    if (params.cursor !== undefined) cleanParams.cursor = params.cursor;
    if (params.application_id !== undefined) cleanParams.application_id = params.application_id;
    if (params.tenant_id !== undefined) cleanParams.tenant_id = params.tenant_id;
    if (params.search_key !== undefined && params.search_key !== '') cleanParams.search_key = params.search_key;
    
    return request.requestJSON({
      method: 'get',
      url: '/api/v1/identity/resources',
      params: cleanParams
    });
  }

  // 获取应用下的资源列表
  getApplicationResources(appId: number): Promise<AxiosResponse<IApiResult<IResourceInfo[]>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/resources/application/${appId}`
    });
  }

  // 获取资源信息
  getResource(id: number): Promise<AxiosResponse<IApiResult<IResourceInfo>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/resources/${id}`
    });
  }

  // 创建资源
  createResource(info: ICreateResourceParam): Promise<AxiosResponse<IApiResult<{ resource_id: number }>>> {
    return request.requestJSON({
      method: 'post',
      url: '/api/v1/identity/resources',
      data: info
    });
  }

  // 更新资源
  updateResource(id: number, info: IUpdateResourceParam): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'put',
      url: `/api/v1/identity/resources/${id}`,
      data: info
    });
  }

  // 删除资源
  deleteResource(id: number): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'delete',
      url: `/api/v1/identity/resources/${id}`
    });
  }

  /**
   * 获取当前用户的菜单资源
   */
  getCurrentUserMenus(applicationId: number, tenantId?: number): Promise<AxiosResponse<IApiResult<IResourceInfo[]>>> {
    const params: Record<string, any> = { application_id: applicationId };
    if (tenantId !== undefined) {
      params.tenant_id = tenantId;
    }
    return request.requestJSON({
      method: 'get',
      url: '/api/v1/identity/resources/menus',
      params
    });
  }

  /**
   * 获取当前用户在指定菜单下的子资源（按钮 / 字段 / 数据等），并按类型分类
   */
  getMenuResources(params: {
    application_id: number;
    menu_id: number;
    tenant_id?: number;
  }): Promise<AxiosResponse<IApiResult<IMenuResourcesByType>>> {
    return request.requestJSON({
      method: 'get',
      url: '/api/v1/identity/resources/menu-resources',
      params
    });
  }
}

export const resourceApi = new ResourceApi();

