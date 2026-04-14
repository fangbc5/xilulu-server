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
        <h1 class="text-2xl font-bold m-0">{{ isNew ? '创建资源' : '资源详情' }}</h1>
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
              <n-icon size="18" :component="DocumentTextOutline" />
              <span class="font-semibold text-base">基本信息</span>
            </div>
            <div class="flex gap-2">
              <n-button @click="handleBack">取消</n-button>
              <n-button type="primary" @click="handleSave" :loading="saving">保存</n-button>
            </div>
          </div>
        </template>
      <n-form ref="formRef" :model="form" :rules="rules" label-placement="left" label-width="120">
        <n-grid :cols="2" :x-gap="24" :y-gap="16">
          <n-gi>
            <n-form-item path="application_id" label="所属应用">
              <n-select
                v-model:value="form.application_id"
                :options="applicationOptions"
                placeholder="请选择应用"
                filterable
                :disabled="!isNew"
              />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="code" label="资源代码">
              <n-input v-model:value="form.code" :disabled="!isNew" placeholder="请输入资源代码（唯一标识）" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="name" label="资源名称">
              <n-input v-model:value="form.name" placeholder="请输入资源名称" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="parent_id" label="父级ID">
              <n-input-number v-model:value="form.parent_id" :min="0" placeholder="请输入父级ID（0表示根节点）" style="width: 100%" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="resource_type" label="资源类型">
              <n-select
                v-model:value="form.resource_type"
                :options="resourceTypeOptions"
                placeholder="请选择资源类型"
                clearable
              />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="path" label="路径">
              <n-input v-model:value="form.path" placeholder="请输入路径" />
            </n-form-item>
          </n-gi>
          <n-gi v-if="isNew">
            <n-form-item path="state" label="状态">
              <n-switch v-model:value="form.state" />
            </n-form-item>
          </n-gi>
          <n-gi :span="2">
            <n-form-item path="describe_" label="描述">
              <n-input
                v-model:value="form.describe_"
                type="textarea"
                :rows="3"
                placeholder="请输入资源描述"
              />
            </n-form-item>
          </n-gi>
        </n-grid>
      </n-form>
    </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import {
  NCard,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NSelect,
  NSwitch,
  NButton,
  FormInst,
  useMessage,
  NGrid,
  NGi,
  NIcon,
  NTag
} from 'naive-ui';
import { ArrowBackOutline, DocumentTextOutline } from '@vicons/ionicons5';
import { resourceApi } from '@/api/resource';
import { applicationApi } from '@/api/application';
import { handleApiResult } from '@/utils/request';
import { IApplicationInfo } from '@/types/base';

const message = useMessage();

const route = useRoute();
const router = useRouter();
const formRef = ref<FormInst | null>(null);

const resourceId = route.params.id as string;
const isNew = resourceId === 'new';
const saving = ref(false);
const applications = ref<IApplicationInfo[]>([]);

const resourceTypeOptions = [
  { label: '菜单', value: '20' },
  { label: '按钮', value: '40' },
  { label: '字段', value: '50' },
  { label: '数据', value: '60' }
];

const applicationOptions = computed(() => {
  return applications.value
    .filter(app => app.id !== undefined)
    .map(app => ({
      label: app.name || `应用 ${app.id}`,
      value: app.id!
    }));
});

const form = ref({
  application_id: 0,
  code: '',
  name: '',
  parent_id: 0,
  resource_type: '20', // 默认菜单
  path: '',
  describe_: '',
  state: true
});

const rules = {
  application_id: {
    required: true,
    message: '请选择应用',
    trigger: 'change',
    validator: (_rule: any, value: number) => {
      if (!value || value === 0) {
        return new Error('请选择应用');
      }
      return true;
    }
  },
  code: {
    required: true,
    message: '请输入资源代码',
    trigger: 'blur'
  },
  name: {
    required: true,
    message: '请输入资源名称',
    trigger: 'blur'
  },
  parent_id: {
    required: true,
    message: '请输入父级ID',
    trigger: 'blur',
    validator: (_rule: any, value: number) => {
      if (value === undefined || value === null || isNaN(value)) {
        return new Error('请输入父级ID');
      }
      return true;
    }
  }
};

const loadApplications = async () => {
  try {
    const response = await applicationApi.getApplicationList({ page_size: 100 });
    const data = handleApiResult(response);
    if (data) {
      applications.value = data.list || [];
    }
  } catch (error: any) {
    console.error('加载应用列表失败', error);
  }
};

const loadResource = async () => {
  if (isNew) return;
  try {
    const response = await resourceApi.getResource(Number(resourceId));
    const data = handleApiResult(response);
    if (data) {
      form.value = {
        application_id: data.application_id,
        code: data.code || '',
        name: data.name || '',
        parent_id: data.parent_id || 0,
        resource_type: data.resource_type || '20',
        path: data.path || '',
        describe_: data.describe_ || '',
        state: data.state ?? true
      };
    }
  } catch (error: any) {
    message.error(error.message || '加载资源信息失败');
  }
};

const handleBack = () => {
  router.push({ name: 'Resources' });
};

const handleSave = async () => {
  try {
    await formRef.value?.validate();
    saving.value = true;
    if (isNew) {
      const response = await resourceApi.createResource({
        application_id: form.value.application_id,
        code: form.value.code,
        name: form.value.name,
        parent_id: form.value.parent_id,
        resource_type: form.value.resource_type || undefined,
        path: form.value.path || undefined,
        describe_: form.value.describe_ || undefined,
        state: form.value.state
      });
      const data = handleApiResult(response);
      if (data?.resource_id) {
        message.success('创建成功');
        // 创建成功后跳转到编辑页面
        router.replace({ name: 'ResourceDetail', params: { id: data.resource_id } });
      }
    } else {
      await resourceApi.updateResource(Number(resourceId), {
        name: form.value.name,
        path: form.value.path || undefined,
        describe_: form.value.describe_ || undefined
      });
      message.success('保存成功');
      // 编辑模式保存后留在当前页面
      await loadResource();
    }
  } catch (error: any) {
    message.error(error.message || '保存失败');
  } finally {
    saving.value = false;
  }
};

onMounted(() => {
  loadApplications();
  loadResource();
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
</style>

