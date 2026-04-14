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
        <h1 class="text-2xl font-bold m-0">{{ isNew ? '创建套餐' : '套餐详情' }}</h1>
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
              <n-icon size="18" :component="CubeOutline" />
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
            <n-form-item path="name" label="套餐名称">
              <n-input v-model:value="form.name" placeholder="请输入套餐名称" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="type" label="套餐类型">
              <n-select
                v-model:value="form.type"
                :options="typeOptions"
                placeholder="请选择套餐类型"
              />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="price" label="价格">
              <n-input
                v-model:value="form.price"
                placeholder="请输入价格（例如：99.00）"
              />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="billing_cycle" label="计费周期">
              <n-select
                v-model:value="form.billing_cycle"
                :options="billingCycleOptions"
                placeholder="请选择计费周期"
              />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="is_active" label="状态">
              <div class="flex items-center gap-2">
                <n-switch v-model:value="form.is_active" />
                <span :class="form.is_active ? 'text-green-600' : 'text-red-600'" style="line-height: 1;">
                  {{ form.is_active ? '启用' : '禁用' }}
                </span>
              </div>
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="sort_order" label="排序">
              <n-input-number
                v-model:value="form.sort_order"
                :min="0"
                placeholder="请输入排序值"
                style="width: 100%"
              />
            </n-form-item>
          </n-gi>
          <n-gi :span="2">
            <n-form-item path="description" label="描述">
              <n-input
                v-model:value="form.description"
                type="textarea"
                :rows="3"
                placeholder="请输入套餐描述"
              />
            </n-form-item>
          </n-gi>
        </n-grid>
      </n-form>
    </n-card>

    <!-- 使用 Tabs 组织功能（仅编辑模式） -->
    <n-tabs v-if="!isNew" v-model:value="activeTab" type="line" animated class="detail-tabs">
      <n-tab-pane name="entitlements" tab="套餐权益">
        <n-card class="tab-content-card" :bordered="false">
          <template #header>
            <div class="flex justify-between items-center">
              <div class="flex items-center gap-2">
                <n-icon size="18" :component="GiftOutline" />
                <span class="font-semibold text-base">套餐权益</span>
              </div>
              <n-button type="primary" @click="handleAddEntitlement">添加权益</n-button>
            </div>
          </template>
          <n-data-table
            :columns="entitlementColumns"
            :data="entitlements"
            :loading="entitlementsLoading"
          />
        </n-card>
      </n-tab-pane>
    </n-tabs>
  </div>

  <!-- 添加/编辑权益对话框 -->
  <n-modal v-model:show="showEntitlementDialog" preset="dialog" :title="isEditEntitlement ? '编辑权益' : '添加权益'">
    <n-form
      ref="entitlementFormRef"
      :model="entitlementForm"
      :rules="entitlementRules"
      label-placement="left"
      label-width="100"
    >
      <n-form-item path="entitlement_key" label="权益键">
        <n-input v-model:value="entitlementForm.entitlement_key" placeholder="例如: max_users" />
      </n-form-item>
      <n-form-item path="entitlement_value" label="权益值">
        <n-input v-model:value="entitlementForm.entitlement_value" placeholder="例如: 100" />
      </n-form-item>
      <n-form-item path="value_type" label="值类型">
        <n-select
          v-model:value="entitlementForm.value_type"
          :options="valueTypeOptions"
          placeholder="请选择值类型"
        />
      </n-form-item>
      <n-form-item path="description" label="描述">
        <n-input
          v-model:value="entitlementForm.description"
          type="textarea"
          :rows="2"
          placeholder="请输入描述"
        />
      </n-form-item>
    </n-form>
    <template #action>
      <div class="flex gap-2 justify-end">
        <n-button @click="showEntitlementDialog = false">取消</n-button>
        <n-button type="primary" @click="handleSaveEntitlement" :loading="savingEntitlement">保存</n-button>
      </div>
    </template>
  </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h } from 'vue';
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
  NSwitch,
  NButton as NButtonComponent,
  NTooltip
} from 'naive-ui';
import {
  ArrowBackOutline,
  CubeOutline,
  GiftOutline,
  TrashOutline,
  PencilOutline
} from '@vicons/ionicons5';
import { planApi, IPlanEntitlementInfo, ICreatePlanRequest, IUpdatePlanRequest, ICreatePlanEntitlementRequest, IUpdatePlanEntitlementRequest } from '@/api/plan';
import { handleApiResult } from '@/utils/request';

const message = useMessage();
const dialog = useDialog();

const route = useRoute();
const router = useRouter();
const formRef = ref<FormInst | null>(null);
const entitlementFormRef = ref<FormInst | null>(null);

const planId = route.params.id as string;
const isNew = planId === 'new';
const saving = ref(false);
const activeTab = ref<string>((route.params.tab as string) || 'entitlements');

const form = ref<ICreatePlanRequest & { id?: number }>({
  name: '',
  type: '',
  price: '0.00',
  billing_cycle: '',
  description: '',
  is_active: true,
  sort_order: 0
});

const typeOptions = [
  { label: '个人版', value: 'personal' },
  { label: '团队版', value: 'team' },
  { label: '企业版', value: 'enterprise' }
];

const billingCycleOptions = [
  { label: '月付', value: 'monthly' },
  { label: '季付', value: 'quarterly' },
  { label: '年付', value: 'yearly' },
  { label: '一次性', value: 'one-time' }
];

const rules: any = {
  name: {
    required: true,
    message: '请输入套餐名称',
    trigger: 'blur'
  },
  type: {
    required: true,
    message: '请选择套餐类型',
    trigger: 'change'
  },
  price: {
    required: true,
    message: '请输入价格',
    trigger: 'blur'
  },
  billing_cycle: {
    required: true,
    message: '请选择计费周期',
    trigger: 'change'
  }
};

// 权益相关
const entitlementsLoading = ref(false);
const entitlements = ref<IPlanEntitlementInfo[]>([]);
const showEntitlementDialog = ref(false);
const isEditEntitlement = ref(false);
const savingEntitlement = ref(false);
const entitlementForm = ref<ICreatePlanEntitlementRequest & { id?: number }>({
  plan_id: 0,
  entitlement_key: '',
  entitlement_value: '',
  value_type: 'number',
  description: ''
});

const valueTypeOptions = [
  { label: '数字', value: 'number' },
  { label: '字符串', value: 'string' },
  { label: '布尔值', value: 'boolean' }
];

const entitlementRules = {
  entitlement_key: {
    required: true,
    message: '请输入权益键',
    trigger: 'blur'
  },
  entitlement_value: {
    required: true,
    message: '请输入权益值',
    trigger: 'blur'
  },
  value_type: {
    required: true,
    message: '请选择值类型',
    trigger: 'change'
  }
};

const entitlementColumns: DataTableColumns<IPlanEntitlementInfo> = [
  { title: 'ID', key: 'id', width: 80 },
  { title: '权益键', key: 'entitlement_key', width: 150 },
  { title: '权益值', key: 'entitlement_value', width: 120 },
  {
    title: '值类型',
    key: 'value_type',
    width: 100,
    render: (row) => {
      const typeMap: Record<string, { label: string; type: string }> = {
        number: { label: '数字', type: 'info' },
        string: { label: '字符串', type: 'success' },
        boolean: { label: '布尔值', type: 'warning' }
      };
      const typeInfo = typeMap[row.value_type] || { label: row.value_type, type: 'default' };
      return h(NTag, { type: typeInfo.type as any }, { default: () => typeInfo.label });
    }
  },
  { title: '描述', key: 'description', ellipsis: { tooltip: true } },
  {
    title: '操作',
    key: 'actions',
    width: 100,
    fixed: 'right',
    render: (row) => {
      return h('div', { class: 'flex gap-1' }, [
        h(
          NTooltip,
          { trigger: 'hover', placement: 'top' },
          {
            trigger: () => h(
              NButtonComponent,
              {
                size: 'small',
                type: 'primary',
                onClick: () => handleEditEntitlement(row),
                quaternary: true,
                circle: true
              },
              {
                icon: () => h(NIcon, { component: PencilOutline, size: 16 })
              }
            ),
            default: () => '编辑'
          }
        ),
        h(
          NTooltip,
          { trigger: 'hover', placement: 'top' },
          {
            trigger: () => h(
              NButtonComponent,
              {
                size: 'small',
                type: 'error',
                onClick: () => handleDeleteEntitlement(row.id!),
                quaternary: true,
                circle: true
              },
              {
                icon: () => h(NIcon, { component: TrashOutline, size: 16 })
              }
            ),
            default: () => '删除'
          }
        )
      ]);
    }
  }
];

const loadPlan = async () => {
  if (isNew) return;
  try {
    const response = await planApi.getPlan(Number(planId));
    const data = handleApiResult(response);
    if (data) {
      form.value = {
        id: data.id,
        name: data.name,
        type: data.type,
        price: String(data.price || '0.00'),
        billing_cycle: data.billing_cycle,
        description: data.description || '',
        is_active: data.is_active ?? true,
        sort_order: data.sort_order || 0
      };
    }
  } catch (error: any) {
    message.error(error.message || '加载套餐信息失败');
  }
};

const loadEntitlements = async () => {
  if (isNew) return;
  entitlementsLoading.value = true;
  try {
    const response = await planApi.getPlanEntitlements(Number(planId));
    const data = handleApiResult(response);
    entitlements.value = data || [];
  } catch (error: any) {
    message.error(error.message || '加载权益列表失败');
  } finally {
    entitlementsLoading.value = false;
  }
};

const handleBack = () => {
  router.push({ name: 'Plans' });
};

const handleSave = async () => {
  if (!formRef.value) return;
  try {
    await formRef.value.validate();
    saving.value = true;
    if (isNew) {
      const response = await planApi.createPlan(form.value as ICreatePlanRequest);
      const data = handleApiResult(response);
      if (data) {
        message.success('创建成功');
        router.push({ name: 'PlanDetail', params: { id: data.plan_id } });
      }
    } else {
      const updateData: IUpdatePlanRequest = {
        name: form.value.name,
        type: form.value.type,
        price: form.value.price,
        billing_cycle: form.value.billing_cycle,
        description: form.value.description,
        is_active: form.value.is_active,
        sort_order: form.value.sort_order
      };
      await planApi.updatePlan(Number(planId), updateData);
      message.success('更新成功');
      await loadPlan();
    }
  } catch (error: any) {
    message.error(error.message || '保存失败');
  } finally {
    saving.value = false;
  }
};

const handleAddEntitlement = () => {
  isEditEntitlement.value = false;
  entitlementForm.value = {
    plan_id: Number(planId),
    entitlement_key: '',
    entitlement_value: '',
    value_type: 'number',
    description: ''
  };
  showEntitlementDialog.value = true;
};

const handleEditEntitlement = (entitlement: IPlanEntitlementInfo) => {
  isEditEntitlement.value = true;
  entitlementForm.value = {
    id: entitlement.id,
    plan_id: entitlement.plan_id,
    entitlement_key: entitlement.entitlement_key,
    entitlement_value: entitlement.entitlement_value,
    value_type: entitlement.value_type,
    description: entitlement.description || ''
  };
  showEntitlementDialog.value = true;
};

const handleSaveEntitlement = async () => {
  if (!entitlementFormRef.value) return;
  try {
    await entitlementFormRef.value.validate();
    savingEntitlement.value = true;
    if (isEditEntitlement.value && entitlementForm.value.id) {
      // 更新权益
      const updateData: IUpdatePlanEntitlementRequest = {
        entitlement_key: entitlementForm.value.entitlement_key,
        entitlement_value: entitlementForm.value.entitlement_value,
        value_type: entitlementForm.value.value_type,
        description: entitlementForm.value.description || undefined
      };
      await planApi.updatePlanEntitlement(entitlementForm.value.id, updateData);
      message.success('更新成功');
    } else {
      // 创建权益
      await planApi.createPlanEntitlement(entitlementForm.value);
      message.success('添加成功');
    }
    showEntitlementDialog.value = false;
    await loadEntitlements();
  } catch (error: any) {
    message.error(error.message || '保存失败');
  } finally {
    savingEntitlement.value = false;
  }
};

const handleDeleteEntitlement = async (id: number) => {
  dialog.warning({
    title: '确认删除',
    content: '确定要删除该权益吗？',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await planApi.deletePlanEntitlement(id);
        message.success('删除成功');
        await loadEntitlements();
      } catch (error: any) {
        message.error(error.message || '删除失败');
      }
    }
  });
};

onMounted(async () => {
  await loadPlan();
  if (!isNew) {
    await loadEntitlements();
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
