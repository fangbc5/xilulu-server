import { AxiosResponse } from 'axios';
import request from '@/utils/request';
import { IApiResult, IApplicationInfo, IPageResult } from '@/types/base';

export interface ICreateApplicationParam {
  app_key: string; // 应用标识（必填）
  name: string;
  type?: string; // 应用类型：10-自建应用 20-第三方应用
  app_secret?: string; // 应用秘钥
  version?: string; // 版本
  redirect?: string; // 重定向地址
  url?: string; // 应用地址
  introduce?: string; // 简介
  remark?: string; // 备注
  is_general?: boolean; // 是否公共应用
  is_visible?: boolean; // 是否可见
  sort_value?: number; // 排序
}

export interface IUpdateApplicationParam {
  name?: string;
  type?: string; // 应用类型
  version?: string; // 版本
  redirect?: string; // 重定向地址
  url?: string; // 应用地址
  introduce?: string; // 简介
  remark?: string; // 备注
  is_general?: boolean; // 是否公共应用
  is_visible?: boolean; // 是否可见
  sort_value?: number; // 排序
}

class ApplicationApi {
  // 获取应用列表（分页）
  getApplicationList(params: {
    page_size?: number;
    cursor?: number;
    search_key?: string;
  }): Promise<AxiosResponse<IApiResult<IPageResult<IApplicationInfo>>>> {
    // 过滤掉 undefined 值
    const cleanParams: Record<string, any> = {};
    if (params.page_size !== undefined) cleanParams.page_size = params.page_size;
    if (params.cursor !== undefined) cleanParams.cursor = params.cursor;
    if (params.search_key !== undefined) cleanParams.search_key = params.search_key;
    
    return request.requestJSON({
      method: 'get',
      url: '/api/v1/identity/applications',
      params: cleanParams
    });
  }

  // 获取应用信息
  getApplication(id: number): Promise<AxiosResponse<IApiResult<IApplicationInfo>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/applications/${id}`
    });
  }

  // 创建应用
  createApplication(info: ICreateApplicationParam): Promise<AxiosResponse<IApiResult<{ application_id: number }>>> {
    return request.requestJSON({
      method: 'post',
      url: '/api/v1/identity/applications',
      data: info
    });
  }

  // 更新应用
  updateApplication(id: number, info: IUpdateApplicationParam): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'put',
      url: `/api/v1/identity/applications/${id}`,
      data: info
    });
  }

  // 删除应用
  deleteApplication(id: number): Promise<AxiosResponse<IApiResult<boolean>>> {
    return request.requestJSON({
      method: 'delete',
      url: `/api/v1/identity/applications/${id}`
    });
  }
}

export const applicationApi = new ApplicationApi();

