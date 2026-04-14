<template>
  <div class="user-detail-container">
    <!-- 页面头部 -->
    <div class="page-header">
      <div class="flex items-center gap-3">
        <n-button quaternary circle @click="handleBack">
          <template #icon>
            <n-icon :component="ArrowBackOutline" />
          </template>
        </n-button>
        <h1 class="text-2xl font-bold m-0">{{ isNew ? '创建用户' : '用户详情' }}</h1>
        <n-tag v-if="!isNew && form.username" type="info" size="small" round>{{ form.username }}</n-tag>
      </div>
    </div>
    
    <!-- 内容区域 -->
    <div class="content-wrapper">
      <!-- 基本信息 -->
      <n-card class="info-card" :bordered="false">
        <template #header>
          <div class="card-header">
            <div class="flex items-center gap-2">
              <n-icon size="18" :component="PersonOutline" />
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
            <n-form-item path="username" label="用户名">
              <n-input v-model:value="form.username" :disabled="!isNew" placeholder="请输入用户名" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item v-if="isNew" path="password" label="密码">
              <n-input
                v-model:value="form.password"
                type="password"
                show-password-on="click"
                placeholder="请输入密码（6-20位）"
              />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="email" label="邮箱">
              <n-input v-model:value="form.email" placeholder="请输入邮箱" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item path="mobile" label="手机号">
              <n-input v-model:value="form.mobile" placeholder="请输入手机号" />
            </n-form-item>
          </n-gi>
          <n-gi :span="2">
            <n-form-item path="nick_name" label="昵称">
              <n-input v-model:value="form.nick_name" placeholder="请输入昵称" />
            </n-form-item>
          </n-gi>
        </n-grid>
      </n-form>
    </n-card>
    <n-tabs v-if="!isNew" type="line" animated class="detail-tabs">
      <n-tab-pane name="password" tab="密码管理">
        <n-card :bordered="false" class="tab-content-card">
          <div class="password-management">
            <div class="mb-4 flex gap-3">
              <n-button type="primary" @click="showChangePasswordDialog = true">
                <template #icon>
                  <n-icon :component="LockClosedOutline" />
                </template>
                修改密码
              </n-button>
              <n-button @click="showResetPasswordDialog = true">
                <template #icon>
                  <n-icon :component="RefreshOutline" />
                </template>
                重置密码
              </n-button>
            </div>
            <n-text depth="3" class="text-sm">您可以修改用户密码或重置用户密码</n-text>
          </div>
        </n-card>
      </n-tab-pane>
      
      <n-tab-pane name="tenants" tab="租户管理">
        <n-card :bordered="false" class="tab-content-card">
          <template #header>
            <div class="flex justify-between items-center">
              <div class="flex items-center gap-2">
                <n-icon size="18" :component="BusinessOutline" />
                <span class="font-semibold text-base">用户所属租户</span>
              </div>
              <n-button type="primary" size="small" @click="showAddTenantDialog = true">
                <template #icon>
                  <n-icon :component="AddOutline" />
                </template>
                添加租户
              </n-button>
            </div>
          </template>
          <n-data-table
            :columns="tenantColumns"
            :data="userTenants"
            :loading="tenantsLoading"
            :single-line="false"
            :max-height="400"
          />
        </n-card>
      </n-tab-pane>
    </n-tabs>
    </div>

    <!-- 修改密码对话框 -->
    <n-modal v-model:show="showChangePasswordDialog" preset="dialog" title="修改密码">
      <n-form ref="changePasswordFormRef" :model="changePasswordForm" :rules="changePasswordRules">
        <n-form-item path="old_password" label="原密码">
          <n-input
            v-model:value="changePasswordForm.old_password"
            type="password"
            show-password-on="click"
            placeholder="请输入原密码"
          />
        </n-form-item>
        <n-form-item path="new_password" label="新密码">
          <n-input
            v-model:value="changePasswordForm.new_password"
            type="password"
            show-password-on="click"
            placeholder="请输入新密码"
          />
        </n-form-item>
        <n-form-item path="confirm_password" label="确认密码">
          <n-input
            v-model:value="changePasswordForm.confirm_password"
            type="password"
            show-password-on="click"
            placeholder="请再次输入新密码"
          />
        </n-form-item>
      </n-form>
      <template #action>
        <n-button @click="showChangePasswordDialog = false">取消</n-button>
        <n-button type="primary" @click="handleChangePassword">确定</n-button>
      </template>
    </n-modal>

    <!-- 重置密码对话框 -->
    <n-modal v-model:show="showResetPasswordDialog" preset="dialog" title="重置密码">
      <n-form ref="resetPasswordFormRef" :model="resetPasswordForm" :rules="resetPasswordRules">
        <n-form-item path="new_password" label="新密码">
          <n-input
            v-model:value="resetPasswordForm.new_password"
            type="password"
            show-password-on="click"
            placeholder="请输入新密码"
          />
        </n-form-item>
        <n-form-item path="confirm_password" label="确认密码">
          <n-input
            v-model:value="resetPasswordForm.confirm_password"
            type="password"
            show-password-on="click"
            placeholder="请再次输入新密码"
          />
        </n-form-item>
      </n-form>
      <template #action>
        <n-button @click="showResetPasswordDialog = false">取消</n-button>
        <n-button type="primary" @click="handleResetPassword">确定</n-button>
      </template>
    </n-modal>

    <!-- 添加租户对话框 -->
    <n-modal v-model:show="showAddTenantDialog" preset="dialog" title="添加租户">
      <n-form ref="addTenantFormRef" :model="addTenantForm" :rules="addTenantRules">
        <n-form-item path="tenant_id" label="选择租户">
          <n-select
            v-model:value="addTenantForm.tenant_id"
            :options="tenantOptions"
            placeholder="请选择租户"
            filterable
            :loading="tenantsLoading"
          />
        </n-form-item>
      </n-form>
      <template #action>
        <n-button @click="showAddTenantDialog = false">取消</n-button>
        <n-button type="primary" @click="handleAddTenant">确定</n-button>
      </template>
    </n-modal>
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
} from 'naive-ui';
import { ArrowBackOutline, PersonOutline, LockClosedOutline, RefreshOutline, BusinessOutline, AddOutline } from '@vicons/ionicons5';
import { userApi, IUserTenantInfo } from '@/api/user';
import { tenantApi } from '@/api/tenant';
import { handleApiResult } from '@/utils/request';
import { ITenantInfo } from '@/types/base';

const message = useMessage();
const dialog = useDialog();

const route = useRoute();
const router = useRouter();
const formRef = ref<FormInst | null>(null);
const changePasswordFormRef = ref<FormInst | null>(null);
const resetPasswordFormRef = ref<FormInst | null>(null);
const addTenantFormRef = ref<FormInst | null>(null);

const userId = route.params.id as string;
const isNew = userId === 'new';
const saving = ref(false);

const form = ref({
  username: '',
  password: '',
  email: '',
  mobile: '',
  nick_name: ''
});

const rules = {
  username: {
    required: true,
    message: '请输入用户名',
    trigger: 'blur'
  },
  password: {
    required: isNew,
    message: '请输入密码',
    trigger: 'blur',
    min: 6,
    max: 20
  }
};

// 密码管理
const showChangePasswordDialog = ref(false);
const showResetPasswordDialog = ref(false);
const changePasswordForm = ref({
  old_password: '',
  new_password: '',
  confirm_password: ''
});
const resetPasswordForm = ref({
  new_password: '',
  confirm_password: ''
});

const changePasswordRules = {
  old_password: {
    required: true,
    message: '请输入原密码',
    trigger: 'blur'
  },
  new_password: {
    required: true,
    message: '请输入新密码',
    trigger: 'blur',
    min: 6,
    max: 20
  },
  confirm_password: {
    required: true,
    message: '请确认新密码',
    trigger: 'blur',
    validator: (_rule: any, value: string) => {
      if (value !== changePasswordForm.value.new_password) {
        return new Error('两次输入的密码不一致');
      }
      return true;
    }
  }
};

const resetPasswordRules = {
  new_password: {
    required: true,
    message: '请输入新密码',
    trigger: 'blur',
    min: 6,
    max: 20
  },
  confirm_password: {
    required: true,
    message: '请确认新密码',
    trigger: 'blur',
    validator: (_rule: any, value: string) => {
      if (value !== resetPasswordForm.value.new_password) {
        return new Error('两次输入的密码不一致');
      }
      return true;
    }
  }
};

// 租户管理
const showAddTenantDialog = ref(false);
const userTenants = ref<IUserTenantInfo[]>([]);
const tenantsLoading = ref(false);
const allTenants = ref<ITenantInfo[]>([]);
const addTenantForm = ref({
  tenant_id: null as number | null
});

const addTenantRules = {
  tenant_id: {
    type: 'number' as const,
    required: true,
    message: '请选择租户',
    trigger: ['blur', 'change']
  }
};

const tenantOptions = computed(() => {
  // 过滤掉已经添加的租户
  const addedTenantIds = new Set(userTenants.value.map(ut => ut.tenant_id));
  return allTenants.value
    .filter(t => t.id && !addedTenantIds.has(t.id))
    .map(t => ({
      label: t.name,
      value: t.id
    }));
});

const tenantColumns: DataTableColumns<IUserTenantInfo> = [
  { title: '租户ID', key: 'tenant_id', width: 100 },
  {
    title: '租户名称',
    key: 'tenant_name',
    width: 200,
    render: (row) => {
      const tenant = allTenants.value.find(t => t.id === row.tenant_id);
      return tenant?.name || `租户 ${row.tenant_id}`;
    }
  },
  {
    title: '角色',
    key: 'is_owner',
    width: 100,
    render: (row) => {
      const isOwner = row.is_owner === 1;
      return h(NTag, { type: isOwner ? 'success' : 'default' }, {
        default: () => isOwner ? '所有者' : '成员'
      });
    }
  },
  {
    title: '是否默认',
    key: 'is_owner',
    width: 100,
    render: (row) => {
      return row.is_owner === 1 ? h(NTag, { type: 'info' }, { default: () => '是' }) : '-';
    }
  },
  {
    title: '加入时间',
    key: 'join_time',
    width: 180,
    render: (row) => {
      return row.join_time ? new Date(row.join_time).toLocaleString('zh-CN') : '-';
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 200,
    fixed: 'right',
    render: (row) => {
      return h('div', { class: 'flex gap-2' }, [
        h(
          NButton,
          {
            size: 'small',
            type: row.is_owner === 1 ? 'default' : 'primary',
            onClick: () => handleSetDefaultTenant(row.tenant_id),
            disabled: row.is_owner === 1
          },
          { default: () => '设为默认' }
        ),
        h(
          NButton,
          {
            size: 'small',
            type: 'error',
            onClick: () => handleRemoveTenant(row.tenant_id)
          },
          { default: () => '移除' }
        )
      ]);
    }
  }
];

const loadUser = async () => {
  if (isNew) return;
  try {
    // 先加载所有租户列表（用于显示租户名称）
    await loadAllTenants();
    
    // 然后并行加载用户信息和用户租户关系
    const [userResponse] = await Promise.all([
      userApi.getUser(Number(userId)),
      loadUserTenants()
    ]);
    
    const data = handleApiResult(userResponse);
    if (data) {
      form.value = {
        username: data.username || '',
        password: '',
        email: data.email || '',
        mobile: data.mobile || '',
        nick_name: data.nick_name || ''
      };
    }
  } catch (error: any) {
    message.error(error.message || '加载用户信息失败');
  }
};

const loadUserTenants = async () => {
  if (isNew) return;
  tenantsLoading.value = true;
  try {
    const response = await userApi.getUserTenants(Number(userId));
    const data = handleApiResult(response);
    userTenants.value = data || [];
  } catch (error: any) {
    message.error(error.message || '加载租户列表失败');
  } finally {
    tenantsLoading.value = false;
  }
};

const loadAllTenants = async () => {
  try {
    const response = await tenantApi.getTenantList({ page_size: 100 });
    const data = handleApiResult(response);
    allTenants.value = data?.list || [];
  } catch (error: any) {
    console.error('加载租户列表失败', error);
  }
};

const handleBack = () => {
  router.push({ name: 'Users' });
};

const handleSave = async () => {
  try {
    await formRef.value?.validate();
    saving.value = true;
    if (isNew) {
      const response = await userApi.createUser({
        username: form.value.username,
        password: form.value.password,
        email: form.value.email || undefined,
        mobile: form.value.mobile || undefined,
        nick_name: form.value.nick_name || undefined
      });
      const data = handleApiResult(response);
      if (data?.user_id) {
        message.success('创建成功');
        // 创建成功后跳转到编辑页面
        router.replace({ name: 'UserDetail', params: { id: data.user_id } });
      }
    } else {
      await userApi.updateUser(Number(userId), {
        username: form.value.username,
        email: form.value.email,
        mobile: form.value.mobile,
        nick_name: form.value.nick_name
      });
      message.success('保存成功');
      // 编辑模式保存后留在当前页面
      await loadUser();
    }
  } catch (error: any) {
    message.error(error.message || '保存失败');
  } finally {
    saving.value = false;
  }
};

const handleChangePassword = async () => {
  try {
    await changePasswordFormRef.value?.validate();
    await userApi.changePassword(Number(userId), {
      old_password: changePasswordForm.value.old_password,
      new_password: changePasswordForm.value.new_password
    });
    message.success('密码修改成功');
    showChangePasswordDialog.value = false;
    changePasswordForm.value = {
      old_password: '',
      new_password: '',
      confirm_password: ''
    };
  } catch (error: any) {
    message.error(error.message || '密码修改失败');
  }
};

const handleResetPassword = async () => {
  try {
    await resetPasswordFormRef.value?.validate();
    await userApi.resetPassword(Number(userId), resetPasswordForm.value.new_password);
    message.success('密码重置成功');
    showResetPasswordDialog.value = false;
    resetPasswordForm.value = {
      new_password: '',
      confirm_password: ''
    };
  } catch (error: any) {
    message.error(error.message || '密码重置失败');
  }
};

const handleAddTenant = async () => {
  try {
    await addTenantFormRef.value?.validate();
    if (!addTenantForm.value.tenant_id) {
      message.error('请选择租户');
      return;
    }
    await userApi.addUserToTenant(Number(userId), addTenantForm.value.tenant_id);
    message.success('添加成功');
    showAddTenantDialog.value = false;
    addTenantForm.value.tenant_id = null;
    await loadUserTenants();
  } catch (error: any) {
    message.error(error.message || '添加失败');
  }
};

const handleRemoveTenant = async (tenantId: number) => {
  dialog.warning({
    title: '确认移除',
    content: '确定要从该租户中移除用户吗？',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await userApi.removeUserFromTenant(Number(userId), tenantId);
        message.success('移除成功');
        await loadUserTenants();
      } catch (error: any) {
        message.error(error.message || '移除失败');
      }
    }
  });
};

const handleSetDefaultTenant = async (tenantId: number) => {
  try {
    await userApi.setDefaultTenant(Number(userId), tenantId);
    message.success('设置成功');
    await loadUserTenants();
  } catch (error: any) {
    message.error(error.message || '设置失败');
  }
};

onMounted(() => {
  loadUser();
});
</script>

<style scoped>
.user-detail-container {
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
  margin-top: 0;
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

.password-management {
  padding: 4px 0;
}
</style>

