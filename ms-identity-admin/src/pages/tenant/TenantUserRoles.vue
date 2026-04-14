<template>
  <div>
    <!-- 页面头部 -->
    <div class="flex items-center justify-between mb-4">
      <div class="flex items-center gap-3">
        <n-button quaternary @click="handleBack">
          <template #icon>
            <n-icon :component="ArrowBackOutline" />
          </template>
        </n-button>
        <h1 class="text-2xl font-bold m-0">用户角色管理</h1>
        <n-tag v-if="userInfo.username" type="info" size="small">{{ userInfo.username }}</n-tag>
        <n-tag v-if="tenantInfo.name" type="success" size="small">{{ tenantInfo.name }}</n-tag>
      </div>
      <div class="flex gap-2">
        <n-button @click="handleBack">返回</n-button>
        <n-button type="primary" @click="showAssignRoleModal = true">
          <template #icon>
            <n-icon :component="AddOutline" />
          </template>
          分配角色
        </n-button>
      </div>
    </div>

    <!-- 用户信息卡片 -->
    <n-card class="mb-4">
      <template #header>
        <div class="flex items-center gap-2">
          <n-icon size="20" :component="PeopleOutline" />
          <span>用户信息</span>
        </div>
      </template>
      <n-descriptions label-placement="left" :column="3" bordered>
        <n-descriptions-item label="用户ID">{{ userInfo.id || '-' }}</n-descriptions-item>
        <n-descriptions-item label="用户名">{{ userInfo.username || '-' }}</n-descriptions-item>
        <n-descriptions-item label="邮箱">{{ userInfo.email || '-' }}</n-descriptions-item>
        <n-descriptions-item label="手机号">{{ userInfo.mobile || '-' }}</n-descriptions-item>
        <n-descriptions-item label="昵称">{{ userInfo.nick_name || '-' }}</n-descriptions-item>
        <n-descriptions-item label="状态">
          <n-tag :type="userInfo.state === 1 ? 'success' : 'error'">
            {{ userInfo.state === 1 ? '启用' : '禁用' }}
          </n-tag>
        </n-descriptions-item>
      </n-descriptions>
    </n-card>

    <!-- 已分配角色列表 -->
    <n-card>
      <template #header>
        <div class="flex items-center gap-2">
          <n-icon size="20" :component="ShieldCheckmarkOutline" />
          <span>已分配角色</span>
        </div>
      </template>
      <n-data-table
        :columns="roleColumns"
        :data="userRoles"
        :loading="rolesLoading"
        :bordered="false"
      />
    </n-card>

    <!-- 分配角色 Modal -->
    <n-modal
      v-model:show="showAssignRoleModal"
      preset="dialog"
      title="分配角色"
      positive-text="确定"
      negative-text="取消"
      @positive-click="handleAssignRole"
      :mask-closable="false"
      style="width: 600px"
    >
      <n-form ref="assignRoleFormRef" :model="assignRoleForm" :rules="assignRoleRules">
        <n-form-item path="role_ids" label="选择角色">
          <n-select
            v-model:value="assignRoleForm.role_ids"
            :options="availableRoleOptions"
            placeholder="请选择角色（可多选）"
            filterable
            multiple
            :loading="availableRolesLoading"
          />
        </n-form-item>
      </n-form>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import {
  NCard,
  NButton,
  NIcon,
  NTag,
  NDataTable,
  DataTableColumns,
  useMessage,
  useDialog,
  NModal,
  NForm,
  NFormItem,
  NSelect,
  FormInst,
  NDescriptions,
  NDescriptionsItem
} from 'naive-ui';
import {
  ArrowBackOutline,
  ShieldCheckmarkOutline,
  PeopleOutline,
  AddOutline,
  TrashOutline
} from '@vicons/ionicons5';
import { userApi, IUserRoleInfo } from '@/api/user';
import { roleApi } from '@/api/role';
import { tenantApi } from '@/api/tenant';
import { handleApiResult } from '@/utils/request';
import { IUserInfo, IRoleInfo, ITenantInfo } from '@/types/base';

const message = useMessage();
const dialog = useDialog();
const route = useRoute();
const router = useRouter();
const assignRoleFormRef = ref<FormInst | null>(null);

const userId = Number(route.params.userId);
const tenantId = Number(route.params.tenantId);

const userInfo = ref<IUserInfo>({});
const tenantInfo = ref<Partial<ITenantInfo>>({});
const userRoles = ref<IUserRoleInfo[]>([]);
const rolesLoading = ref(false);

const showAssignRoleModal = ref(false);
const assignRoleForm = ref({
  role_ids: [] as number[]
});

const assignRoleRules = {
  role_ids: {
    required: true,
    type: 'array' as const,
    message: '请至少选择一个角色',
    trigger: 'change',
    validator: (_rule: any, value: number[]) => {
      if (!value || value.length === 0) {
        return new Error('请至少选择一个角色');
      }
      return true;
    }
  }
};

const availableRoles = ref<IRoleInfo[]>([]);
const availableRolesLoading = ref(false);
const availableRoleOptions = computed(() => {
  // 过滤掉已分配的角色
  const assignedRoleIds = new Set(userRoles.value.map(r => r.role_id));
  return availableRoles.value
    .filter(role => role.id !== undefined && !assignedRoleIds.has(role.id))
    .map(role => ({
      label: `${role.name} (${role.code})`,
      value: role.id!
    }));
});

const roleColumns: DataTableColumns<IUserRoleInfo> = [
  { title: 'ID', key: 'id', width: 80 },
  { title: '角色ID', key: 'role_id', width: 100 },
  { title: '角色编码', key: 'role_code', width: 150 },
  {
    title: '创建时间',
    key: 'created_at',
    width: 180,
    render: (row) => {
      return row.created_at ? new Date(row.created_at).toLocaleString('zh-CN') : '-';
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
          onClick: () => handleRemoveRole(row.role_id),
          quaternary: true,
          circle: true
        },
        {
          icon: () => h(NIcon, { component: TrashOutline, size: 16 })
        }
      );
    }
  }
];

const loadUserInfo = async () => {
  try {
    const response = await userApi.getUser(userId);
    const data = handleApiResult(response);
    if (data) {
      userInfo.value = data;
    }
  } catch (error: any) {
    console.error('加载用户信息失败', error);
  }
};

const loadTenantInfo = async () => {
  try {
    const response = await tenantApi.getTenant(tenantId);
    const data = handleApiResult(response);
    if (data) {
      tenantInfo.value = data;
    }
  } catch (error: any) {
    console.error('加载租户信息失败', error);
  }
};

const loadUserRoles = async () => {
  rolesLoading.value = true;
  try {
    const response = await userApi.getUserRoles(userId, tenantId);
    const data = handleApiResult(response);
    userRoles.value = data || [];
  } catch (error: any) {
    message.error(error.message || '加载用户角色失败');
    userRoles.value = [];
  } finally {
    rolesLoading.value = false;
  }
};

const loadAvailableRoles = async () => {
  availableRolesLoading.value = true;
  try {
    const response = await roleApi.getTenantRoles(tenantId);
    const data = handleApiResult(response);
    availableRoles.value = data || [];
  } catch (error: any) {
    console.error('加载可用角色失败', error);
    message.error('加载可用角色失败: ' + (error.message || '未知错误'));
    availableRoles.value = [];
  } finally {
    availableRolesLoading.value = false;
  }
};

const handleBack = () => {
  router.push({ name: 'TenantDetail', params: { id: tenantId } });
};

const handleAssignRole = async () => {
  try {
    await assignRoleFormRef.value?.validate();
    if (!assignRoleForm.value.role_ids || assignRoleForm.value.role_ids.length === 0) {
      message.error('请至少选择一个角色');
      return;
    }

    // 使用批量分配接口
    await userApi.batchAssignRolesToUser(userId, tenantId, assignRoleForm.value.role_ids);
    message.success(`成功分配 ${assignRoleForm.value.role_ids.length} 个角色`);
    showAssignRoleModal.value = false;
    assignRoleForm.value.role_ids = [];
    await loadUserRoles();
  } catch (error: any) {
    message.error(error.message || '角色分配失败');
  }
};

const handleRemoveRole = (roleId: number) => {
  dialog.warning({
    title: '移除角色',
    content: '确定从该用户移除此角色吗？',
    positiveText: '确定',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        await userApi.removeRoleFromUser(userId, tenantId, roleId);
        message.success('角色移除成功');
        await loadUserRoles();
      } catch (error: any) {
        message.error(error.message || '角色移除失败');
      }
    }
  });
};

onMounted(async () => {
  await Promise.all([
    loadUserInfo(),
    loadTenantInfo(),
    loadUserRoles(),
    loadAvailableRoles()
  ]);
});
</script>

