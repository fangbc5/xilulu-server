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
        <h1 class="text-2xl font-bold m-0">{{ isNew ? '创建应用' : '应用详情' }}</h1>
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
              <n-icon size="18" :component="AppsOutline" />
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
            <n-form-item path="app_key" label="应用标识">
              <n-input v-model:value="form.app_key" :disabled="!isNew" placeholder="请输入应用标识（唯一标识）" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="name" label="应用名称">
              <n-input v-model:value="form.name" placeholder="请输入应用名称" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="type" label="应用类型">
              <n-select
                v-model:value="form.type"
                :options="typeOptions"
                placeholder="请选择应用类型"
                clearable
              />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="version" label="版本">
              <n-input v-model:value="form.version" placeholder="请输入版本号" />
            </n-form-item>
          </n-gi>
          <n-gi v-if="isNew">
            <n-form-item path="app_secret" label="应用秘钥">
              <n-input v-model:value="form.app_secret" type="password" show-password-on="click" placeholder="请输入应用秘钥" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="url" label="应用地址">
              <n-input v-model:value="form.url" placeholder="请输入应用访问地址" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="redirect" label="重定向地址">
              <n-input v-model:value="form.redirect" placeholder="请输入重定向地址" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="is_general" label="是否公共应用">
              <n-switch v-model:value="form.is_general" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="is_visible" label="是否可见">
              <n-switch v-model:value="form.is_visible" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="sort_value" label="排序值">
              <n-input-number v-model:value="form.sort_value" :min="1" placeholder="请输入排序值" style="width: 100%" />
            </n-form-item>
          </n-gi>
          <n-gi :span="2">
            <n-form-item path="introduce" label="简介">
              <n-input
                v-model:value="form.introduce"
                type="textarea"
                :rows="3"
                placeholder="请输入应用简介"
              />
            </n-form-item>
          </n-gi>
          <n-gi :span="2">
            <n-form-item path="remark" label="备注">
              <n-input
                v-model:value="form.remark"
                type="textarea"
                :rows="3"
                placeholder="请输入备注信息"
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
import { ref, onMounted } from 'vue';
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
import { ArrowBackOutline, AppsOutline } from '@vicons/ionicons5';
import { applicationApi } from '@/api/application';
import { handleApiResult } from '@/utils/request';

const message = useMessage();

const route = useRoute();
const router = useRouter();
const formRef = ref<FormInst | null>(null);

const applicationId = route.params.id as string;
const isNew = applicationId === 'new';
const saving = ref(false);

const typeOptions = [
  { label: '自建应用', value: '10' },
  { label: '第三方应用', value: '20' }
];

const form = ref({
  app_key: '',
  name: '',
  type: '10', // 默认自建应用
  app_secret: '',
  version: '',
  redirect: '',
  url: '',
  introduce: '',
  remark: '',
  is_general: false,
  is_visible: true,
  sort_value: 1
});

const rules = {
  app_key: {
    required: true,
    message: '请输入应用标识',
    trigger: 'blur'
  },
  name: {
    required: true,
    message: '请输入应用名称',
    trigger: 'blur'
  }
};

const loadApplication = async () => {
  if (isNew) return;
  try {
    const response = await applicationApi.getApplication(Number(applicationId));
    const data = handleApiResult(response);
    if (data) {
      form.value = {
        app_key: data.app_key || '',
        name: data.name || '',
        type: data.type || '10',
        app_secret: '', // 秘钥不返回
        version: data.version || '',
        redirect: data.redirect || '',
        url: data.url || '',
        introduce: data.introduce || '',
        remark: data.remark || '',
        is_general: data.is_general ?? false,
        is_visible: data.is_visible ?? true,
        sort_value: data.sort_value || 1
      };
    }
  } catch (error: any) {
    message.error(error.message || '加载应用信息失败');
  }
};

const handleBack = () => {
  router.push({ name: 'Applications' });
};

const handleSave = async () => {
  try {
    await formRef.value?.validate();
    saving.value = true;
    if (isNew) {
      const response = await applicationApi.createApplication({
        app_key: form.value.app_key,
        name: form.value.name,
        type: form.value.type || undefined,
        app_secret: form.value.app_secret || undefined,
        version: form.value.version || undefined,
        redirect: form.value.redirect || undefined,
        url: form.value.url || undefined,
        introduce: form.value.introduce || undefined,
        remark: form.value.remark || undefined,
        is_general: form.value.is_general,
        is_visible: form.value.is_visible,
        sort_value: form.value.sort_value
      });
      const data = handleApiResult(response);
      if (data?.application_id) {
        message.success('创建成功');
        // 创建成功后跳转到编辑页面
        router.replace({ name: 'ApplicationDetail', params: { id: data.application_id } });
      }
    } else {
      await applicationApi.updateApplication(Number(applicationId), {
        name: form.value.name,
        type: form.value.type || undefined,
        version: form.value.version || undefined,
        redirect: form.value.redirect || undefined,
        url: form.value.url || undefined,
        introduce: form.value.introduce || undefined,
        remark: form.value.remark || undefined,
        is_general: form.value.is_general,
        is_visible: form.value.is_visible,
        sort_value: form.value.sort_value
      });
      message.success('保存成功');
      // 编辑模式保存后留在当前页面
      await loadApplication();
    }
  } catch (error: any) {
    message.error(error.message || '保存失败');
  } finally {
    saving.value = false;
  }
};

onMounted(() => {
  loadApplication();
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
