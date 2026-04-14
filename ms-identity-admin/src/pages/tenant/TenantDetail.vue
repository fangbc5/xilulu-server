<template>
  <div class="detail-container">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="flex items-center gap-3">
        <n-button quaternary circle @click="handleBack">
          <template #icon>
            <n-icon :component="ArrowBackOutline" />
          </template>
        </n-button>
        <h1 class="text-2xl font-bold m-0">{{ isNew ? '创建租户' : '租户详情' }}</h1>
        <n-tag v-if="!isNew && form.name" type="info" size="small" round>{{ form.name }}</n-tag>
      </div>
    </div>

    <!-- 内容区域 -->
    <div class="content-wrapper">
    <!-- 基本信息 -->
      <n-card class="info-card" :bordered="false">
      <template #header>
          <div class="card-header">
        <div class="flex items-center gap-2">
              <n-icon size="18" :component="BusinessOutline" />
              <span class="font-semibold text-base">基本信息</span>
            </div>
            <div class="flex gap-2">
              <n-button @click="handleBack">取消</n-button>
              <n-button type="primary" @click="handleSave" :loading="saving">保存</n-button>
            </div>
        </div>
      </template>
      <n-form ref="formRef" :model="form" :rules="rules" label-placement="left" label-width="100">
        <n-grid :cols="2" :x-gap="24" :y-gap="16">
          <n-gi>
            <n-form-item path="name" label="租户名称">
              <n-input v-model:value="form.name" placeholder="请输入租户名称" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="contact_name" label="联系人">
              <n-input v-model:value="form.contact_name" placeholder="请输入联系人" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="contact_mobile" label="联系电话">
              <n-input v-model:value="form.contact_mobile" placeholder="请输入联系电话" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="website" label="网站">
              <n-input v-model:value="form.website" placeholder="可选" />
            </n-form-item>
          </n-gi>
          <n-gi v-if="isNew">
            <n-form-item path="package_id" label="套餐">
              <n-select
                v-model:value="form.package_id"
                :options="planOptions"
                placeholder="请选择套餐"
                filterable
                :loading="plansLoading"
              />
            </n-form-item>
          </n-gi>
          <n-gi v-if="isNew">
            <n-form-item path="account_count" label="账户数">
              <n-input-number
                v-model:value="form.account_count"
                :min="1"
                :max="10000"
                placeholder="请输入账户数"
                style="width: 100%"
              />
            </n-form-item>
          </n-gi>
          <n-gi v-if="isNew">
            <n-form-item path="expire_time" label="过期时间">
              <n-date-picker
                v-model:value="form.expire_time"
                type="datetime"
                placeholder="请选择过期时间"
                style="width: 100%"
                :is-date-disabled="(timestamp: number) => timestamp < Date.now()"
              />
            </n-form-item>
          </n-gi>
          <n-gi v-if="!isNew">
            <n-form-item path="status" label="状态">
              <n-select
                v-model:value="form.status"
                :options="statusOptions"
                placeholder="请选择状态"
              />
            </n-form-item>
          </n-gi>
        </n-grid>
      </n-form>
    </n-card>

    <!-- 使用 Tabs 组织功能（仅编辑模式） -->
    <n-tabs v-if="!isNew" type="line" animated class="detail-tabs">
      <n-tab-pane name="subscription" tab="套餐信息">
        <n-card class="tab-content-card" :bordered="false">
          <template #header>
            <div class="flex justify-between items-center">
              <div class="flex items-center gap-2">
                <n-icon size="20" :component="CubeOutline" />
                <span>套餐订阅信息</span>
              </div>
              <n-button type="primary" @click="handleAddSubscription">
                <template #icon>
                  <n-icon :component="AddOutline" />
                </template>
                订阅套餐
              </n-button>
            </div>
          </template>
          <n-spin :show="subscriptionLoading">
            <div v-if="subscriptions.length > 0" class="space-y-4">
              <n-list>
                <n-list-item v-for="(sub, index) in subscriptions" :key="sub.id || index" style="margin-bottom: 20px;">
                  <n-card :title="sub.plan?.name || `套餐 #${sub.plan_id}`" size="small">
                    <template #header-extra>
                      <n-button
                        v-if="sub.status === 'active'"
                        size="small"
                        type="error"
                        quaternary
                        @click="handleCancelSubscription(sub)"
                      >
                        退订
                      </n-button>
                    </template>
                    <n-descriptions label-placement="left" :column="2" bordered>
                      <n-descriptions-item label="套餐名称">
                        <n-tag type="info">{{ sub.plan?.name || '-' }}</n-tag>
                      </n-descriptions-item>
                      <n-descriptions-item label="套餐类型">
                        {{ sub.plan?.type || '-' }}
                      </n-descriptions-item>
                      <n-descriptions-item label="价格">
                        {{ sub.plan?.price || '0.00' }} 元
                      </n-descriptions-item>
                      <n-descriptions-item label="计费周期">
                        {{ sub.plan?.billing_cycle || '-' }}
                      </n-descriptions-item>
                      <n-descriptions-item label="描述" :span="2">
                        {{ sub.plan?.description || '-' }}
                      </n-descriptions-item>
                      <n-descriptions-item label="订阅状态">
                        <n-tag :type="getSubscriptionStatusType(sub.status)">
                          {{ getSubscriptionStatusText(sub.status) }}
                        </n-tag>
                      </n-descriptions-item>
                      <n-descriptions-item label="自动续费">
                        <n-tag :type="sub.auto_renew ? 'success' : 'default'">
                          {{ sub.auto_renew ? '是' : '否' }}
                        </n-tag>
                      </n-descriptions-item>
                      <n-descriptions-item label="开始时间">
                        {{ sub.start_at ? new Date(sub.start_at).toLocaleString('zh-CN') : '-' }}
                      </n-descriptions-item>
                      <n-descriptions-item label="过期时间">
                        {{ sub.expire_at ? new Date(sub.expire_at).toLocaleString('zh-CN') : '-' }}
                      </n-descriptions-item>
                    </n-descriptions>
                  </n-card>
                </n-list-item>
              </n-list>
            </div>
            <n-empty v-else description="暂无套餐订阅信息" />
          </n-spin>
        </n-card>
      </n-tab-pane>
      
      <n-tab-pane name="applications" tab="应用管理">
        <n-card class="tab-content-card" :bordered="false">
          <template #header>
            <div class="flex justify-between items-center">
              <div class="flex items-center gap-2">
                <n-icon size="20" :component="AppsOutline" />
                <span>租户应用列表</span>
              </div>
              <n-button type="primary" size="small" @click="showAddApplicationDialog = true">
                <template #icon>
                  <n-icon :component="AddOutline" />
                </template>
                添加应用
              </n-button>
            </div>
          </template>
          <n-data-table
            :columns="applicationColumns"
            :data="tenantApplications"
            :loading="applicationsLoading"
            :bordered="false"
            :max-height="400"
          />
        </n-card>
      </n-tab-pane>

      <n-tab-pane name="users" tab="用户列表">
        <n-card class="tab-content-card" :bordered="false">
          <template #header>
            <div class="flex justify-between items-center">
              <div class="flex items-center gap-2">
                <n-icon size="20" :component="PeopleOutline" />
                <span>租户用户列表</span>
              </div>
              <div class="flex gap-2">
                <n-input
                  v-model:value="userSearchKey"
                  placeholder="搜索用户名、邮箱、手机号、昵称"
                  clearable
                  style="width: 300px"
                  @keyup.enter="loadUsers"
                />
                <n-button @click="loadUsers">搜索</n-button>
              </div>
            </div>
          </template>
          <n-data-table
            :columns="userColumns"
            :data="tenantUsers"
            :loading="usersLoading"
            :pagination="userPagination"
            remote
            :bordered="false"
          />
        </n-card>
      </n-tab-pane>
    </n-tabs>

    <!-- 订阅套餐对话框 -->
    <n-modal v-model:show="showSubscriptionDialog" preset="dialog" title="订阅套餐">
      <n-form ref="subscriptionFormRef" :model="subscriptionForm" :rules="subscriptionRules" label-placement="left" label-width="100">
        <n-form-item path="plan_id" label="选择套餐">
          <n-select
            v-model:value="subscriptionForm.plan_id"
            :options="subscriptionPlanOptions"
            placeholder="请选择套餐或搜索套餐名称"
            filterable
            remote
            :loading="subscriptionPlansLoading"
            @search="handlePlanSearch"
            @focus="handlePlanFocus"
            @update:value="handlePlanChange"
            clearable
          />
        </n-form-item>
        <n-form-item v-if="selectedPlanBillingCycle === 'one_time'" path="start_at" label="开始时间">
          <n-date-picker
            v-model:value="subscriptionForm.start_at"
            type="datetime"
            format="yyyy-MM-dd HH:mm:ss"
            value-format="timestamp"
            placeholder="请选择开始时间"
            clearable
          />
        </n-form-item>
        <n-form-item v-if="selectedPlanBillingCycle === 'one_time'" path="expire_at" label="过期时间">
          <n-date-picker
            v-model:value="subscriptionForm.expire_at"
            type="datetime"
            format="yyyy-MM-dd HH:mm:ss"
            value-format="timestamp"
            placeholder="请选择过期时间"
            clearable
          />
        </n-form-item>
        <n-form-item v-if="selectedPlanBillingCycle !== 'one_time'" path="auto_renew" label="自动续费">
          <div class="flex items-center gap-2">
            <n-switch
              v-model:value="subscriptionForm.auto_renew"
              :disabled="selectedPlanBillingCycle === 'forever'"
            />
            <span class="text-gray-500 text-sm">
              <template v-if="selectedPlanBillingCycle === 'forever'">
                无限期套餐默认不自动续费，无法修改
              </template>
              <template v-else>
                订阅后立即生效，过期时间根据套餐计费周期自动计算
              </template>
            </span>
          </div>
        </n-form-item>
        <n-form-item v-if="selectedPlanBillingCycle === 'one_time'">
          <span class="text-gray-500 text-sm">一次性套餐需要手动设置开始时间和过期时间</span>
        </n-form-item>
      </n-form>
      <template #action>
        <div class="flex gap-2 justify-end">
          <n-button @click="showSubscriptionDialog = false">取消</n-button>
          <n-button type="primary" @click="handleSaveSubscription" :loading="savingSubscription">确定</n-button>
        </div>
      </template>
    </n-modal>

    <!-- 添加应用对话框 -->
    <n-modal v-model:show="showAddApplicationDialog" preset="dialog" title="添加应用">
      <n-form ref="addApplicationFormRef" :model="addApplicationForm" :rules="addApplicationRules">
        <n-form-item path="application_id" label="选择应用">
          <n-select
            v-model:value="addApplicationForm.application_id"
            :options="applicationOptions"
            placeholder="请选择应用"
            filterable
            :loading="allApplicationsLoading"
          />
        </n-form-item>
      </n-form>
      <template #action>
        <n-button @click="showAddApplicationDialog = false">取消</n-button>
        <n-button type="primary" @click="handleAddApplication">确定</n-button>
      </template>
    </n-modal>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import {
  NCard,
  NForm,
  NFormItem,
  NInput,
  NButton,
  FormInst,
  useMessage,
  useDialog,
  NModal,
  NDataTable,
  DataTableColumns,
  NSelect,
  NTag,
  NTabs,
  NTabPane,
  NGrid,
  NGi,
  NIcon,
  NInputNumber,
  NDatePicker,
  NSpin,
  NDescriptions,
  NDescriptionsItem,
  NEmpty,
  NList,
  NListItem
} from 'naive-ui';
import {
  ArrowBackOutline,
  BusinessOutline,
  AppsOutline,
  AddOutline,
  TrashOutline,
  CubeOutline,
  PeopleOutline,
  PencilOutline,
  ShieldCheckmarkOutline
} from '@vicons/ionicons5';
import { tenantApi } from '@/api/tenant';
import { applicationApi } from '@/api/application';
import { planApi, ITenantSubscriptionInfo, ICreateTenantSubscriptionRequest } from '@/api/plan';
import { userApi } from '@/api/user';
import { handleApiResult } from '@/utils/request';
import { IApplicationInfo, IPlanInfo, IUserInfo } from '@/types/base';

const message = useMessage();
const dialog = useDialog();

const route = useRoute();
const router = useRouter();
const formRef = ref<FormInst | null>(null);
const addApplicationFormRef = ref<FormInst | null>(null);

const tenantId = route.params.id as string;
const isNew = tenantId === 'new';
const saving = ref(false);

const form = ref({
  name: '',
  contact_name: '',
  contact_mobile: '',
  website: '',
  package_id: null as number | null,
  account_count: 10,
  expire_time: null as number | null,
  status: 0
});

const rules: any = {
  name: {
    required: true,
    message: '请输入租户名称',
    trigger: 'blur'
  },
  contact_name: {
    required: true,
    message: '请输入联系人',
    trigger: 'blur'
  },
  package_id: {
    required: isNew,
    type: 'number' as const,
    message: '请选择套餐',
    trigger: ['blur', 'change']
  },
  account_count: {
    required: isNew,
    type: 'number' as const,
    message: '请输入账户数',
    trigger: 'blur'
  },
  expire_time: {
    required: isNew,
    type: 'number' as const,
    message: '请选择过期时间',
    trigger: 'change'
  }
};

const statusOptions = [
  { label: '正常', value: 0 },
  { label: '停用', value: 1 }
];

// 套餐相关
const plansLoading = ref(false);
const allPlans = ref<IPlanInfo[]>([]);
const planOptions = computed(() => {
  return allPlans.value.map(p => ({
    label: `${p.name} (${p.type})`,
    value: p.id
  }));
});

// 当前套餐和订阅信息
const subscriptionLoading = ref(false);
const subscriptions = ref<ITenantSubscriptionInfo[]>([]);
const showSubscriptionDialog = ref(false);
const savingSubscription = ref(false);

// 订阅状态映射函数
const getSubscriptionStatusText = (status?: string): string => {
  switch (status) {
    case 'active':
      return '激活';
    case 'scheduled':
      return '已预约';
    case 'expired':
      return '已过期';
    case 'canceled':
      return '已取消';
    default:
      return status || '未知';
  }
};

const getSubscriptionStatusType = (status?: string): 'success' | 'warning' | 'error' | 'default' => {
  switch (status) {
    case 'active':
      return 'success';
    case 'scheduled':
      return 'warning';
    case 'expired':
      return 'error';
    case 'canceled':
      return 'default';
    default:
      return 'default';
  }
};
const subscriptionFormRef = ref<FormInst | null>(null);
const subscriptionForm = ref({
  plan_id: null as number | null,
  auto_renew: false,
  start_at: null as number | null,
  expire_at: null as number | null
});

// 存储套餐列表的详细信息，用于获取billing_cycle
const subscriptionPlansMap = ref<Map<number, IPlanInfo>>(new Map());
const selectedPlanBillingCycle = computed(() => {
  if (!subscriptionForm.value.plan_id) return '';
  const planId = Number(subscriptionForm.value.plan_id); // 确保是数字类型
  const plan = subscriptionPlansMap.value.get(planId);
  if (!plan) {
    return '';
  }
  const cycle = (plan.billing_cycle || '').trim().toLowerCase();
  return cycle;
});

const subscriptionRules = {
  plan_id: {
    required: true,
    message: '请选择套餐',
    trigger: ['change', 'blur'],
    validator: (_rule: any, value: number | null) => {
      if (!value) {
        return new Error('请选择套餐');
      }
      return true;
    }
  },
  start_at: {
    required: false,
    trigger: ['change', 'blur'],
    validator: (_rule: any, value: number | null) => {
      if (selectedPlanBillingCycle.value === 'one_time' && !value) {
        return new Error('一次性套餐需要设置开始时间');
      }
      // one_time 套餐时，开始时间必须早于结束时间，且整体时长不能超过 10 年
      if (
        selectedPlanBillingCycle.value === 'one_time' &&
        value &&
        subscriptionForm.value.expire_at &&
        subscriptionForm.value.expire_at <= value
      ) {
        return new Error('开始时间必须早于过期时间');
      }
      return true;
    }
  },
  expire_at: {
    required: false,
    trigger: ['change', 'blur'],
    validator: (_rule: any, value: number | null) => {
      if (selectedPlanBillingCycle.value === 'one_time' && !value) {
        return new Error('一次性套餐需要设置过期时间');
      }
      if (
        selectedPlanBillingCycle.value === 'one_time' &&
        subscriptionForm.value.start_at &&
        value &&
        value <= subscriptionForm.value.start_at
      ) {
        return new Error('过期时间必须晚于开始时间');
      }
      // 校验时长不能超过 10 年（按 365 天*10 粗略计算）
      if (
        selectedPlanBillingCycle.value === 'one_time' &&
        subscriptionForm.value.start_at &&
        value
      ) {
        const tenYearsMs = 10 * 365 * 24 * 60 * 60 * 1000;
        if (value - subscriptionForm.value.start_at > tenYearsMs) {
          return new Error('一次性套餐时长不能超过 10 年');
        }
      }
      return true;
    }
  }
};

// 订阅套餐选择相关
const subscriptionPlansLoading = ref(false);
const subscriptionPlanOptions = ref<Array<{ label: string; value: number }>>([]);
const subscriptionPlanSearchKey = ref('');

const loadSubscriptionPlans = async (searchKey: string = '') => {
  subscriptionPlansLoading.value = true;
  try {
    const response = await planApi.getPlanList({
      page_size: 20,
      // 这里使用 page=1 对应的 cursor=1，避免后端将 0 视为非法页码
      cursor: 1,
      search_key: searchKey || undefined,
      // 传递租户 ID，用于过滤已订阅的套餐
      exclude_subscribed_tenant_id: isNew ? undefined : Number(tenantId)
    });
    const data = handleApiResult(response);
    if (data?.list) {
      // 更新套餐映射
      data.list.forEach(p => {
        // 修复：id 为 0 时，if (p.id) 会失败，需要显式检查
        if (p.id !== undefined && p.id !== null) {
          const planId = Number(p.id); // 确保是数字类型
          subscriptionPlansMap.value.set(planId, p);
        }
      });
      subscriptionPlanOptions.value = data.list
        .filter(p => p.id !== undefined && p.id !== null) // 过滤掉没有 id 的套餐
        .map(p => ({
          label: `${p.name} (${p.type}) - ${p.price || '0.00'}元/${p.billing_cycle || ''}`,
          value: Number(p.id!) // 确保 value 是数字类型
        }));
    }
  } catch (error: any) {
    console.error('加载套餐列表失败', error);
    message.error('加载套餐列表失败');
  } finally {
    subscriptionPlansLoading.value = false;
  }
};

const handlePlanSearch = (query: string) => {
  subscriptionPlanSearchKey.value = query;
  loadSubscriptionPlans(query);
};

const handlePlanFocus = () => {
  // 如果选项为空且不在加载中，则加载套餐列表
  if (subscriptionPlanOptions.value.length === 0 && !subscriptionPlansLoading.value) {
    loadSubscriptionPlans();
  }
};

const handlePlanChange = (planId: number | null) => {
  if (!planId) {
    // 清空时重置表单
    subscriptionForm.value.plan_id = null;
    return;
  }
  
  // 确保 planId 是数字类型
  const numericPlanId = Number(planId);
  
  // 显式设置表单的 plan_id（确保值被正确更新）
  subscriptionForm.value.plan_id = numericPlanId;
  
  // 确保 map 中有数据
  if (!subscriptionPlansMap.value.has(numericPlanId)) {
    // 如果 map 中没有，尝试重新加载
    loadSubscriptionPlans().then(() => {
      // 重新加载后再次处理
      const cycle = selectedPlanBillingCycle.value;
      applyPlanCycleLogic(cycle);
    });
    return;
  }
  
  const cycle = selectedPlanBillingCycle.value;
  applyPlanCycleLogic(cycle);

  // 选择套餐后触发表单验证（只校验套餐字段）
  if (subscriptionFormRef.value) {
    subscriptionFormRef.value.restoreValidation();
    subscriptionFormRef.value.validate();
  }
};

const applyPlanCycleLogic = (cycle: string) => {
  if (cycle === 'forever') {
    // 无限期套餐自动续费固定为 false
    subscriptionForm.value.auto_renew = false;
    subscriptionForm.value.start_at = null;
    subscriptionForm.value.expire_at = null;
  } else if (cycle === 'one_time') {
    // 一次性套餐需要用户手动选择开始和结束时间
    subscriptionForm.value.start_at = null;
    subscriptionForm.value.expire_at = null;
  } else {
    // 其他周期型套餐，清理时间字段
    subscriptionForm.value.start_at = null;
    subscriptionForm.value.expire_at = null;
  }
};

// 应用管理
const showAddApplicationDialog = ref(false);
const tenantApplications = ref<IApplicationInfo[]>([]); // 改为 ApplicationInfo，因为后端返回的是应用信息
const applicationsLoading = ref(false);
const allApplications = ref<IApplicationInfo[]>([]);
const allApplicationsLoading = ref(false);
const addApplicationForm = ref({
  application_id: null as number | null
});

const addApplicationRules = {
  application_id: {
    type: 'number' as const,
    required: true,
    message: '请选择应用',
    trigger: ['blur', 'change']
  }
};

const applicationOptions = computed(() => {
  // 过滤掉已经添加的应用
  const addedApplicationIds = new Set(
    tenantApplications.value
      .map(app => app.id)
      .filter((id): id is number => id !== undefined)
  );
  return allApplications.value
    .filter(app => app.id && !addedApplicationIds.has(app.id))
    .map(app => ({
      label: `${app.name || '未知应用'} (${app.id})`,
      value: app.id
    }));
});

const applicationColumns: DataTableColumns<IApplicationInfo> = [
  { title: '应用ID', key: 'id', width: 100 },
  {
    title: '应用名称',
    key: 'name',
    width: 200,
    render: (row) => {
      return row.name || `应用 ${row.id || '-'}`;
    }
  },
  {
    title: '简介',
    key: 'introduce',
    width: 200,
    render: (row) => {
      return row.introduce || '-';
    }
  },
  {
    title: '备注',
    key: 'remark',
    width: 150,
    render: (row) => {
      return row.remark || '-';
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 100,
    fixed: 'right',
    render: (row) => {
      return h(
        NButton,
        {
          size: 'small',
          type: 'error',
          onClick: () => handleRemoveApplication(row.id!),
          quaternary: true
        },
        {
          default: () => '移除',
          icon: () => h(NIcon, { component: TrashOutline })
        }
      );
    }
  }
];

// 用户列表相关
const usersLoading = ref(false);
const tenantUsers = ref<IUserInfo[]>([]);
const userSearchKey = ref('');
const userPagination = ref({
  page: 1,
  pageSize: 10,
  showSizePicker: true,
  pageSizes: [10, 20, 50],
  itemCount: 0,
  pageCount: 0,
  showQuickJumper: true,
  onChange: (page: number) => {
    userPagination.value.page = page;
    loadUsers();
  },
  onUpdatePageSize: (size: number) => {
    userPagination.value.pageSize = size;
    userPagination.value.page = 1;
    loadUsers();
  },
  prefix: ({ itemCount }: { itemCount?: number }) => {
    const count = itemCount || 0;
    const totalPages = count > 0 ? Math.ceil(count / userPagination.value.pageSize) : 0;
    return `共 ${count} 条，共 ${totalPages} 页`;
  }
});

const userColumns: DataTableColumns<IUserInfo> = [
  { title: 'ID', key: 'id', width: 80 },
  { title: '用户名', key: 'username', width: 120 },
  { title: '邮箱', key: 'email', width: 180 },
  { title: '手机号', key: 'mobile', width: 120 },
  { title: '昵称', key: 'nick_name', width: 120 },
  {
    title: '状态',
    key: 'state',
    width: 100,
    render: (row) => {
      return h(
        NTag,
        { type: row.state === 1 ? 'success' : 'error' },
        { default: () => (row.state === 1 ? '启用' : '禁用') }
      );
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 150,
    fixed: 'right',
    render: (row) => {
      return h('div', { class: 'flex gap-1' }, [
        h(
          NButton,
          {
            size: 'small',
            type: 'primary',
            onClick: () => handleEditUser(row.id!),
            quaternary: true,
            circle: true
          },
          {
            icon: () => h(NIcon, { component: PencilOutline, size: 16 })
          }
        ),
        h(
          NButton,
          {
            size: 'small',
            type: 'info',
            onClick: () => handleAssignRoles(row.id!),
            quaternary: true,
            circle: true
          },
          {
            icon: () => h(NIcon, { component: ShieldCheckmarkOutline, size: 16 })
          }
        )
      ]);
    }
  }
];

const loadUsers = async () => {
  if (isNew) return;
  usersLoading.value = true;
  try {
    const response = await userApi.getUserList({
      page_size: userPagination.value.pageSize,
      cursor: userPagination.value.page > 1 ? userPagination.value.page : undefined,
      tenant_id: Number(tenantId),
      search_key: userSearchKey.value || undefined
    });
    const data = handleApiResult(response);
    if (data) {
      tenantUsers.value = data.list || [];
      userPagination.value.itemCount = data.total || 0;
      userPagination.value.pageCount = data.total > 0 ? Math.ceil(data.total / userPagination.value.pageSize) : 0;
    } else {
      tenantUsers.value = [];
      userPagination.value.itemCount = 0;
      userPagination.value.pageCount = 0;
    }
  } catch (error: any) {
    message.error(error.message || '加载用户列表失败');
    tenantUsers.value = [];
    userPagination.value.itemCount = 0;
    userPagination.value.pageCount = 0;
  } finally {
    usersLoading.value = false;
  }
};

const handleEditUser = (userId: number) => {
  router.push({ name: 'UserDetail', params: { id: userId } });
};

const handleAssignRoles = (userId: number) => {
  router.push({ name: 'TenantUserRoles', params: { tenantId, userId } });
};

const loadTenant = async () => {
  if (isNew) return;
  try {
    // 并行加载租户信息和租户应用关系（所有应用列表在 onMounted 中单独加载，避免重复请求）
    const [tenantResponse] = await Promise.all([
      tenantApi.getTenant(Number(tenantId)),
      loadTenantApplications()
    ]);

    const data = handleApiResult(tenantResponse);
    if (data) {
      form.value = {
        name: data.name || '',
        contact_name: data.contact_name || '',
        contact_mobile: data.contact_mobile || '',
        website: data.website || '',
        package_id: data.package_id || null,
        account_count: data.account_count || 10,
        expire_time: data.expire_time ? new Date(data.expire_time).getTime() : null,
        status: data.status ?? 0
      };
      
      // 根据租户ID加载套餐信息和订阅信息（通过订阅获取）
      await loadPlanAndSubscription();
    }
  } catch (error: any) {
    message.error(error.message || '加载租户信息失败');
  }
};

const loadPlanAndSubscription = async () => {
  subscriptionLoading.value = true;
  try {
    // 获取租户所有订阅信息（包含套餐信息）
    const response = await planApi.getTenantSubscriptions(Number(tenantId)).catch((error: any) => {
      // 订阅可能不存在，这是正常的
      console.log('租户订阅信息不存在或加载失败', error);
      return null;
    });
    
    if (response) {
      const data = handleApiResult(response);
      subscriptions.value = data || [];
    }
  } catch (error: any) {
    console.error('加载套餐信息失败', error);
    message.error('加载套餐信息失败: ' + (error.message || '未知错误'));
  } finally {
    subscriptionLoading.value = false;
  }
};

const loadTenantApplications = async () => {
  if (isNew) return;
  applicationsLoading.value = true;
  try {
    const response = await tenantApi.getTenantApplications(Number(tenantId));
    const data = handleApiResult(response);
    tenantApplications.value = data || [];
  } catch (error: any) {
    message.error(error.message || '加载应用列表失败');
  } finally {
    applicationsLoading.value = false;
  }
};

const loadAllApplications = async () => {
  // 对于租户0，后端 /tenants/0/applications 已经返回所有应用；
  // 此时不需要再额外请求一次，直接复用 tenantApplications 即可，避免重复调用接口。
  if (!isNew && Number(tenantId) === 0) {
    allApplications.value = tenantApplications.value;
    return;
  }

  allApplicationsLoading.value = true;
  try {
    // 使用应用列表接口获取所有应用信息
    const response = await applicationApi.getApplicationList({
      page_size: 100,
      cursor: undefined
    });
    const data = handleApiResult(response);
    allApplications.value = data?.list || [];
  } catch (error: any) {
    console.error('加载应用列表失败', error);
    message.error('加载应用列表失败: ' + (error.message || '未知错误'));
    allApplications.value = [];
  } finally {
    allApplicationsLoading.value = false;
  }
};

const loadAllPlans = async () => {
  plansLoading.value = true;
  try {
    // 需要同时传 cursor，后端使用游标分页；这里固定取第 1 页即可
    const response = await planApi.getPlanList({
      page_size: 100,
      cursor: 1
    });
    const data = handleApiResult(response);
    allPlans.value = data?.list || [];
  } catch (error: any) {
    console.error('加载套餐列表失败', error);
  } finally {
    plansLoading.value = false;
  }
};

const handleBack = () => {
  router.push({ name: 'Tenants' });
};

const handleSave = async () => {
  try {
    await formRef.value?.validate();
    saving.value = true;
    if (isNew) {
      if (!form.value.package_id || !form.value.expire_time) {
        message.error('请选择套餐和过期时间');
        saving.value = false;
        return;
      }
      const response = await tenantApi.createTenant({
        name: form.value.name,
        contact_name: form.value.contact_name,
        contact_mobile: form.value.contact_mobile || undefined,
        website: form.value.website || undefined,
        package_id: form.value.package_id,
        expire_time: new Date(form.value.expire_time).toISOString(),
        account_count: form.value.account_count
      });
      const data = handleApiResult(response);
      if (data?.tenant_id) {
        message.success('创建成功');
        // 创建成功后跳转到编辑页面
        router.replace({ name: 'TenantDetail', params: { id: data.tenant_id } });
      }
    } else {
      await tenantApi.updateTenant(Number(tenantId), {
        name: form.value.name,
        contact_name: form.value.contact_name,
        contact_mobile: form.value.contact_mobile || undefined,
        website: form.value.website || undefined,
        status: form.value.status
      });
      message.success('保存成功');
      // 编辑模式保存后留在当前页面
      await loadTenant();
    }
  } catch (error: any) {
    message.error(error.message || '保存失败');
  } finally {
    saving.value = false;
  }
};

const handleAddApplication = async () => {
  try {
    await addApplicationFormRef.value?.validate();
    if (!addApplicationForm.value.application_id) {
      message.error('请选择应用');
      return;
    }
    await tenantApi.addApplicationToTenant(Number(tenantId), addApplicationForm.value.application_id);
    message.success('添加成功');
    showAddApplicationDialog.value = false;
    addApplicationForm.value.application_id = null;
    // 重新加载租户应用列表和所有应用列表（更新选项）
    await Promise.all([
      loadTenantApplications(),
      loadAllApplications()
    ]);
  } catch (error: any) {
    message.error(error.message || '添加失败');
  }
};

const handleAddSubscription = () => {
  subscriptionForm.value = {
    plan_id: null,
    auto_renew: false,
    start_at: null,
    expire_at: null
  };
  // 清空套餐选项，让 focus 事件触发加载
  subscriptionPlanOptions.value = [];
  showSubscriptionDialog.value = true;
};

const handleSaveSubscription = async () => {
  if (!subscriptionFormRef.value) return;
  try {
    await subscriptionFormRef.value.validate();
    savingSubscription.value = true;
    // 构建请求数据，只包含必要的字段
    const requestData: ICreateTenantSubscriptionRequest = {
      tenant_id: Number(tenantId),
      plan_id: Number(subscriptionForm.value.plan_id)
    };
    
    // 如果是一次性套餐，需要设置开始时间和过期时间
    if (selectedPlanBillingCycle.value === 'one_time') {
      if (!subscriptionForm.value.start_at || !subscriptionForm.value.expire_at) {
        message.error('一次性套餐需要设置开始时间和过期时间');
        savingSubscription.value = false;
        return;
      }
      // 将时间戳转换为ISO 8601格式字符串
      requestData.start_at = new Date(subscriptionForm.value.start_at).toISOString();
      requestData.expire_at = new Date(subscriptionForm.value.expire_at).toISOString();
    } else {
      // 非一次性套餐，只有当 auto_renew 为 true 时才添加该字段
      if (subscriptionForm.value.auto_renew) {
        requestData.auto_renew = true;
      }
    }
    
    await planApi.createTenantSubscription(requestData);
    message.success('订阅成功');
    showSubscriptionDialog.value = false;
    await loadPlanAndSubscription();
  } catch (error: any) {
    message.error(error.message || '订阅失败');
  } finally {
    savingSubscription.value = false;
  }
};

const handleCancelSubscription = async (subscription: ITenantSubscriptionInfo) => {
  dialog.warning({
    title: '确认退订',
    content: `确定要退订套餐 "${subscription.plan?.name || '未知套餐'}" 吗？`,
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await planApi.cancelTenantSubscription(Number(tenantId));
        message.success('退订成功');
        await loadPlanAndSubscription();
      } catch (error: any) {
        message.error(error.message || '退订失败');
      }
    }
  });
};

const handleRemoveApplication = async (applicationId: number) => {
  dialog.warning({
    title: '确认移除',
    content: '确定要从该租户中移除应用吗？',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await tenantApi.removeApplicationFromTenant(Number(tenantId), applicationId);
        message.success('移除成功');
        await Promise.all([
          loadTenantApplications(),
          loadAllApplications()
        ]);
      } catch (error: any) {
        message.error(error.message || '移除失败');
      }
    }
  });
};

onMounted(async () => {
  // 顺序执行，确保租户信息和租户应用先加载完成
  await loadAllPlans();
  await loadTenant();
  if (!isNew) {
    await Promise.all([
      loadAllApplications(),
      loadUsers()
    ]);
  }
});
</script>

<style scoped>
.detail-container {
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--n-divider-color);
}

.content-wrapper {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.info-card {
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  border-radius: 8px;
}

.info-card :deep(.n-card-header) {
  padding: 12px 16px;
  border-bottom: 1px solid var(--n-divider-color);
  background: var(--n-color);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.info-card :deep(.n-card-body) {
  padding: 16px;
}

.detail-tabs {
  margin-top: 0;
}

.detail-tabs :deep(.n-tabs-nav) {
  margin-bottom: 0;
  padding: 0 20px;
  background: var(--n-color);
  border-bottom: 1px solid var(--n-divider-color);
}

.detail-tabs :deep(.n-tabs-tab) {
  padding: 12px 20px;
  margin-right: 8px;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

.detail-tabs :deep(.n-tabs-tab--active) {
  color: var(--n-color-primary);
}

.detail-tabs :deep(.n-tabs-bar) {
  height: 2px;
}

.detail-tabs :deep(.n-tab-pane) {
  padding: 0;
}

.tab-content-card {
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  margin-top: 20px;
  border-radius: 8px;
}

.tab-content-card :deep(.n-card-header) {
  padding: 16px 20px;
  border-bottom: 1px solid var(--n-divider-color);
  background: var(--n-color);
}

.tab-content-card :deep(.n-card-body) {
  padding: 20px;
}
</style>
