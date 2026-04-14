import { AxiosResponse } from 'axios';
import request from '@/utils/request';
import { IApiResult, IPlanInfo } from '@/types/base';

export interface ITenantSubscriptionInfo {
  id?: number;
  tenant_id: number;
  plan_id: number;
  status?: string;
  start_at?: string;
  expire_at?: string;
  auto_renew?: boolean;
  plan?: IPlanInfo; // 套餐详细信息
}

export interface ICreateTenantSubscriptionRequest {
  tenant_id: number;
  plan_id: number;
  start_at?: string; // ISO 8601 datetime string, 可选，不提供则由后端自动设置为当前时间
  expire_at?: string; // ISO 8601 datetime string, 可选，不提供则由后端根据套餐计费周期自动计算
  auto_renew?: boolean;
}

export interface IUpdateTenantSubscriptionRequest {
  plan_id: number;
  status?: string;
  start_at?: string; // ISO 8601 datetime string
  expire_at?: string; // ISO 8601 datetime string
  auto_renew?: boolean;
}

export interface IPlanEntitlementInfo {
  id?: number;
  plan_id: number;
  entitlement_key: string;
  entitlement_value: string;
  value_type: string;
  description?: string;
}

export interface ICreatePlanRequest {
  name: string;
  type: string;
  price: string;
  billing_cycle: string;
  description?: string;
  is_active?: boolean;
  sort_order?: number;
}

export interface IUpdatePlanRequest {
  name?: string;
  type?: string;
  price?: string;
  billing_cycle?: string;
  description?: string;
  is_active?: boolean;
  sort_order?: number;
}

export interface ICreatePlanEntitlementRequest {
  plan_id: number;
  entitlement_key: string;
  entitlement_value: string;
  value_type: string;
  description?: string;
}

export interface IUpdatePlanEntitlementRequest {
  entitlement_key?: string;
  entitlement_value?: string;
  value_type?: string;
  description?: string;
}

class PlanApi {
  // 获取套餐列表
  getPlanList(params?: { page_size?: number; cursor?: number; search_key?: string; exclude_subscribed_tenant_id?: number }): Promise<AxiosResponse<IApiResult<{ list: IPlanInfo[]; total: number; cursor?: number; has_next: boolean }>>> {
    return request.requestJSON({
      method: 'get',
      url: '/api/v1/identity/plans',
      params
    });
  }

  // 获取套餐信息
  getPlan(id: number): Promise<AxiosResponse<IApiResult<IPlanInfo>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/plans/${id}`
    });
  }

  // 创建套餐
  createPlan(data: ICreatePlanRequest): Promise<AxiosResponse<IApiResult<{ plan_id: number }>>> {
    return request.requestJSON({
      method: 'post',
      url: '/api/v1/identity/plans',
      data
    });
  }

  // 更新套餐
  updatePlan(id: number, data: IUpdatePlanRequest): Promise<AxiosResponse<IApiResult<void>>> {
    return request.requestJSON({
      method: 'put',
      url: `/api/v1/identity/plans/${id}`,
      data
    });
  }

  // 删除套餐
  deletePlan(id: number): Promise<AxiosResponse<IApiResult<void>>> {
    return request.requestJSON({
      method: 'delete',
      url: `/api/v1/identity/plans/${id}`
    });
  }

  // 获取套餐权益列表
  getPlanEntitlements(planId: number): Promise<AxiosResponse<IApiResult<IPlanEntitlementInfo[]>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/plans/${planId}/entitlements`
    });
  }

  // 创建套餐权益
  createPlanEntitlement(data: ICreatePlanEntitlementRequest): Promise<AxiosResponse<IApiResult<{ plan_id: number }>>> {
    return request.requestJSON({
      method: 'post',
      url: `/api/v1/identity/plans/${data.plan_id}/entitlements`,
      data
    });
  }

  // 更新套餐权益
  updatePlanEntitlement(id: number, data: IUpdatePlanEntitlementRequest): Promise<AxiosResponse<IApiResult<void>>> {
    return request.requestJSON({
      method: 'put',
      url: `/api/v1/identity/plans/entitlements/${id}`,
      data
    });
  }

  // 删除套餐权益
  deletePlanEntitlement(id: number): Promise<AxiosResponse<IApiResult<void>>> {
    return request.requestJSON({
      method: 'delete',
      url: `/api/v1/identity/plans/entitlements/${id}`
    });
  }

  // 获取租户所有订阅信息（包含套餐信息）
  getTenantSubscriptions(tenantId: number): Promise<AxiosResponse<IApiResult<ITenantSubscriptionInfo[]>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/plans/subscriptions/${tenantId}`
    });
  }

  // 获取租户当前激活的订阅信息（包含套餐信息）
  getTenantActiveSubscription(tenantId: number): Promise<AxiosResponse<IApiResult<ITenantSubscriptionInfo | null>>> {
    return request.requestJSON({
      method: 'get',
      url: `/api/v1/identity/plans/subscriptions/${tenantId}/active`
    });
  }

  // 创建租户订阅
  createTenantSubscription(data: ICreateTenantSubscriptionRequest): Promise<AxiosResponse<IApiResult<{ subscription_id: number }>>> {
    return request.requestJSON({
      method: 'post',
      url: `/api/v1/identity/plans/subscriptions/${data.tenant_id}`,
      data
    });
  }

  // 更新租户订阅
  updateTenantSubscription(tenantId: number, data: IUpdateTenantSubscriptionRequest): Promise<AxiosResponse<IApiResult<void>>> {
    return request.requestJSON({
      method: 'put',
      url: `/api/v1/identity/plans/subscriptions/${tenantId}`,
      data
    });
  }

  // 取消订阅（退订）
  cancelTenantSubscription(tenantId: number): Promise<AxiosResponse<IApiResult<void>>> {
    return request.requestJSON({
      method: 'delete',
      url: `/api/v1/identity/plans/subscriptions/${tenantId}`
    });
  }
}

export const planApi = new PlanApi();

